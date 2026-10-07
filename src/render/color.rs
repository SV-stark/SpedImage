//! Matching rendered colours to the monitor's ICC profile.
//!
//! Decoded images are converted to sRGB (`apply_color_profile`), and the
//! swapchain is an sRGB surface, so on a wide-gamut panel every sRGB colour is
//! shown with the panel's own, more saturated primaries: reds look orange-red,
//! skin tones too warm. Colour-managed viewers fix that with one more step,
//! sRGB -> display.
//!
//! Almost every monitor profile is a matrix/TRC profile: three colorant tags
//! (`rXYZ`, `gXYZ`, `bXYZ`) plus tone curves. The gamut mismatch lives in the
//! colorants, so this module derives a single 3x3 matrix taking linear sRGB to
//! linear display RGB, and the shader applies it as the last step. The tone
//! curve is left to the sRGB surface encode; monitor curves are close to sRGB
//! (or gamma 2.2), and the primaries are what make wide-gamut panels look
//! wrong. LUT-only profiles have no colorant tags and are skipped.
//!
//! Only the display transform runs on the GPU. CPU buffers stay sRGB, so
//! saving, clipboard export and histograms are unaffected.

/// Linear-RGB 3x3 matrix, row-major: `out[r] = sum(m[r][c] * in[c])`.
pub type Mat3 = [[f64; 3]; 3];

/// sRGB colorants adapted to the ICC D50 PCS (Bradford), exactly as the
/// `sRGB IEC61966-2.1` profile stores them. Columns are R, G, B.
const SRGB_TO_XYZ_D50: Mat3 = [
    [0.436_074_7, 0.385_064_9, 0.143_080_4],
    [0.222_504_5, 0.716_878_6, 0.060_616_9],
    [0.013_932_2, 0.097_104_5, 0.714_173_3],
];

fn be_u32(b: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_be_bytes(b.get(at..at + 4)?.try_into().ok()?))
}

fn s15_fixed16(b: &[u8], at: usize) -> Option<f64> {
    Some(i32::from_be_bytes(b.get(at..at + 4)?.try_into().ok()?) as f64 / 65536.0)
}

/// Read an `XYZType` tag body: `'XYZ '`, 4 reserved bytes, then X, Y, Z.
fn xyz_tag(icc: &[u8], offset: usize) -> Option<[f64; 3]> {
    if icc.get(offset..offset + 4)? != b"XYZ " {
        return None;
    }
    Some([
        s15_fixed16(icc, offset + 8)?,
        s15_fixed16(icc, offset + 12)?,
        s15_fixed16(icc, offset + 16)?,
    ])
}

/// The display's linear RGB -> XYZ (D50) matrix from its colorant tags, or
/// `None` for anything that is not an RGB matrix/TRC profile.
pub fn display_rgb_to_xyz(icc: &[u8]) -> Option<Mat3> {
    // Header colour space at offset 16 must be RGB.
    if icc.get(16..20)? != b"RGB " {
        return None;
    }
    let count = be_u32(icc, 128)? as usize;
    // A tag table larger than the file is corrupt; this also caps the loop.
    if count > (icc.len().saturating_sub(132)) / 12 {
        return None;
    }
    let mut cols: [Option<[f64; 3]>; 3] = [None; 3];
    for i in 0..count {
        let entry = 132 + i * 12;
        let sig = icc.get(entry..entry + 4)?;
        let offset = be_u32(icc, entry + 4)? as usize;
        let slot = match sig {
            b"rXYZ" => 0,
            b"gXYZ" => 1,
            b"bXYZ" => 2,
            _ => continue,
        };
        cols[slot] = xyz_tag(icc, offset);
    }
    let [r, g, b] = [cols[0]?, cols[1]?, cols[2]?];
    Some([[r[0], g[0], b[0]], [r[1], g[1], b[1]], [r[2], g[2], b[2]]])
}

fn mul(a: &Mat3, b: &Mat3) -> Mat3 {
    let mut out = [[0.0; 3]; 3];
    for (r, row) in out.iter_mut().enumerate() {
        for (c, cell) in row.iter_mut().enumerate() {
            *cell = (0..3).map(|k| a[r][k] * b[k][c]).sum();
        }
    }
    out
}

fn invert(m: &Mat3) -> Option<Mat3> {
    let det = m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
        - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
        + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0]);
    if det.abs() < 1e-9 || !det.is_finite() {
        return None;
    }
    let inv_det = 1.0 / det;
    let c =
        |r1: usize, c1: usize, r2: usize, c2: usize| m[r1][c1] * m[r2][c2] - m[r1][c2] * m[r2][c1];
    Some([
        [
            c(1, 1, 2, 2) * inv_det,
            -c(0, 1, 2, 2) * inv_det,
            c(0, 1, 1, 2) * inv_det,
        ],
        [
            -c(1, 0, 2, 2) * inv_det,
            c(0, 0, 2, 2) * inv_det,
            -c(0, 0, 1, 2) * inv_det,
        ],
        [
            c(1, 0, 2, 1) * inv_det,
            -c(0, 0, 2, 1) * inv_det,
            c(0, 0, 1, 1) * inv_det,
        ],
    ])
}

/// Linear sRGB -> linear display RGB for the given monitor profile.
///
/// Returns `None` when the profile cannot be used, *or* when it is close
/// enough to sRGB that the conversion would be a no-op, so the common case of
/// an sRGB monitor costs nothing in the shader.
pub fn srgb_to_display_matrix(icc: &[u8]) -> Option<Mat3> {
    let display = display_rgb_to_xyz(icc)?;
    let m = mul(&invert(&display)?, &SRGB_TO_XYZ_D50);
    if m.iter().flatten().any(|v| !v.is_finite()) {
        return None;
    }
    let max_dev = (0..3)
        .flat_map(|r| (0..3).map(move |c| (r, c)))
        .map(|(r, c)| (m[r][c] - if r == c { 1.0 } else { 0.0 }).abs())
        .fold(0.0, f64::max);
    (max_dev > 2e-3).then_some(m)
}

/// The ICC profile Windows has associated with the monitor showing `window`.
#[cfg(windows)]
pub fn window_display_profile(window: &winit::window::Window) -> Option<Vec<u8>> {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::Graphics::Gdi::{GetDC, ReleaseDC};
    use windows::Win32::UI::ColorSystem::GetICMProfileW;
    use windows::core::PWSTR;
    use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

    let hwnd = match window.window_handle().ok()?.as_raw() {
        RawWindowHandle::Win32(h) => HWND(h.hwnd.get() as *mut _),
        _ => return None,
    };
    let path = unsafe {
        // A window DC resolves to the monitor the window is on.
        let hdc = GetDC(Some(hwnd));
        if hdc.is_invalid() {
            return None;
        }
        let mut len: u32 = 0;
        let _ = GetICMProfileW(hdc, &mut len, None);
        let mut buf = vec![0u16; len.max(1) as usize];
        let ok = len > 0 && GetICMProfileW(hdc, &mut len, Some(PWSTR(buf.as_mut_ptr()))).as_bool();
        ReleaseDC(Some(hwnd), hdc);
        if !ok {
            return None;
        }
        let end = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
        String::from_utf16_lossy(&buf[..end])
    };
    std::fs::read(path).ok()
}

/// Display profiles are only queried on Windows for now. macOS (ColorSync)
/// and Linux (colord) would slot in here.
#[cfg(not(windows))]
pub fn window_display_profile(_window: &winit::window::Window) -> Option<Vec<u8>> {
    None
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// Build a minimal RGB matrix/TRC-shaped profile with just the colorants.
    pub(crate) fn profile_with_colorants(cols: Mat3) -> Vec<u8> {
        let mut icc = vec![0u8; 128];
        icc[12..16].copy_from_slice(b"mntr");
        icc[16..20].copy_from_slice(b"RGB ");
        icc[36..40].copy_from_slice(b"acsp");
        let tags = [b"rXYZ", b"gXYZ", b"bXYZ"];
        icc.extend_from_slice(&(tags.len() as u32).to_be_bytes());
        let data_start = 128 + 4 + tags.len() * 12;
        for (i, sig) in tags.iter().enumerate() {
            icc.extend_from_slice(*sig);
            icc.extend_from_slice(&((data_start + i * 20) as u32).to_be_bytes());
            icc.extend_from_slice(&20u32.to_be_bytes());
        }
        for c in 0..3 {
            icc.extend_from_slice(b"XYZ \0\0\0\0");
            for row in &cols {
                icc.extend_from_slice(&((row[c] * 65536.0).round() as i32).to_be_bytes());
            }
        }
        icc
    }

    /// Display P3 colorants adapted to D50, as Apple's profile stores them.
    pub(crate) const P3_D50: Mat3 = [
        [0.515_102, 0.291_965, 0.157_153],
        [0.241_182, 0.692_236, 0.066_582],
        [-0.001_049, 0.041_885, 0.784_378],
    ];

    #[test]
    fn colorants_are_read_back() {
        let icc = profile_with_colorants(P3_D50);
        let m = display_rgb_to_xyz(&icc).unwrap();
        for r in 0..3 {
            for c in 0..3 {
                assert!((m[r][c] - P3_D50[r][c]).abs() < 1e-4);
            }
        }
    }

    #[test]
    fn an_srgb_monitor_needs_no_conversion() {
        assert_eq!(
            srgb_to_display_matrix(&profile_with_colorants(SRGB_TO_XYZ_D50)),
            None
        );
    }

    #[test]
    fn a_wide_gamut_monitor_desaturates_srgb_primaries() {
        let m = srgb_to_display_matrix(&profile_with_colorants(P3_D50)).unwrap();
        // White stays white: each row sums to ~1.
        for row in &m {
            assert!((row.iter().sum::<f64>() - 1.0).abs() < 2e-3, "{row:?}");
        }
        // Pure sRGB red maps to a P3 red below full intensity with some
        // green mixed in, i.e. it is pulled inside the wider gamut. These
        // match the published sRGB->P3 matrix (0.8225, 0.0332, 0.0171).
        let red = [m[0][0], m[1][0], m[2][0]];
        assert!((red[0] - 0.8225).abs() < 3e-3, "{red:?}");
        assert!((red[1] - 0.0332).abs() < 3e-3, "{red:?}");
        assert!((red[2] - 0.0171).abs() < 3e-3, "{red:?}");
    }

    #[test]
    fn non_matrix_or_corrupt_profiles_are_rejected() {
        assert_eq!(display_rgb_to_xyz(&[]), None);
        let mut cmyk = profile_with_colorants(P3_D50);
        cmyk[16..20].copy_from_slice(b"CMYK");
        assert_eq!(display_rgb_to_xyz(&cmyk), None);
        let mut huge_count = profile_with_colorants(P3_D50);
        huge_count[128..132].copy_from_slice(&u32::MAX.to_be_bytes());
        assert_eq!(display_rgb_to_xyz(&huge_count), None);
        // Colorants missing (e.g. a LUT-only profile).
        let truncated = &profile_with_colorants(P3_D50)[..140];
        assert_eq!(display_rgb_to_xyz(truncated), None);
    }

    #[test]
    fn invert_round_trips() {
        let inv = invert(&P3_D50).unwrap();
        let id = mul(&P3_D50, &inv);
        for (r, row) in id.iter().enumerate() {
            for (c, v) in row.iter().enumerate() {
                let want = if r == c { 1.0 } else { 0.0 };
                assert!((v - want).abs() < 1e-9);
            }
        }
        assert_eq!(invert(&[[0.0; 3]; 3]), None);
    }
}
