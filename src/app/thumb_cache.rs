//! Persistent on-disk cache for downsampled RGBA thumbnails.
//!
//! Each entry is keyed by a 64-bit FNV-1a hash of `(path, file_size, mtime,
//! thumb_size)` so modified or replaced files automatically miss the cache.

use std::path::{Path, PathBuf};
use std::sync::Arc;

const MAGIC: &[u8; 8] = b"SPEDTHM1";

fn cache_dir() -> Option<PathBuf> {
    dirs::cache_dir().map(|d| d.join("spedimage").join("thumbs"))
}

fn fnv1a_key(path: &Path, file_size: u64, modified: Option<i64>, thumb_size: u32) -> String {
    const OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;

    let mut hash = OFFSET_BASIS;
    let mut feed = |bytes: &[u8]| {
        for &b in bytes {
            hash ^= b as u64;
            hash = hash.wrapping_mul(PRIME);
        }
    };

    feed(path.to_string_lossy().as_bytes());
    feed(&[0xff]);
    feed(&file_size.to_le_bytes());
    feed(&modified.unwrap_or(0).to_le_bytes());
    feed(&thumb_size.to_le_bytes());

    format!("{hash:016x}.thmb")
}

/// Read a cached thumbnail if present and valid.
pub fn load(
    path: &Path,
    file_size: u64,
    modified: Option<i64>,
    max_dim: u32,
) -> Option<(Arc<Vec<u8>>, u32, u32)> {
    let dir = cache_dir()?;
    let cache_path = dir.join(fnv1a_key(path, file_size, modified, max_dim));
    load_from_file(&cache_path, max_dim)
}

fn load_from_file(cache_path: &Path, max_dim: u32) -> Option<(Arc<Vec<u8>>, u32, u32)> {
    let data = std::fs::read(cache_path).ok()?;
    if data.len() < 16 || &data[0..8] != MAGIC {
        return None;
    }
    let width = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);
    let height = u32::from_le_bytes([data[12], data[13], data[14], data[15]]);
    if width == 0 || height == 0 || width > max_dim || height > max_dim {
        return None;
    }
    let expected = 16usize.checked_add(
        (width as usize)
            .checked_mul(height as usize)?
            .checked_mul(4)?,
    )?;
    if data.len() != expected {
        return None;
    }
    Some((Arc::new(data[16..].to_vec()), width, height))
}

/// Persist a decoded thumbnail atomically on disk.
pub fn store(
    path: &Path,
    file_size: u64,
    modified: Option<i64>,
    max_dim: u32,
    width: u32,
    height: u32,
    rgba: &[u8],
) {
    let Some(dir) = cache_dir() else {
        return;
    };
    let cache_path = dir.join(fnv1a_key(path, file_size, modified, max_dim));
    let _ = store_to_file(&cache_path, width, height, rgba);
}

fn store_to_file(cache_path: &Path, width: u32, height: u32, rgba: &[u8]) -> std::io::Result<()> {
    if let Some(parent) = cache_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut buf = Vec::with_capacity(16 + rgba.len());
    buf.extend_from_slice(MAGIC);
    buf.extend_from_slice(&width.to_le_bytes());
    buf.extend_from_slice(&height.to_le_bytes());
    buf.extend_from_slice(rgba);

    let tmp = cache_path.with_extension(format!("tmp.{}", std::process::id()));
    std::fs::write(&tmp, &buf)?;
    if let Err(e) = std::fs::rename(&tmp, cache_path) {
        let _ = std::fs::remove_file(&tmp);
        return Err(e);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_changes_when_metadata_changes() {
        let p = Path::new("C:/photos/img1.jpg");
        let k1 = fnv1a_key(p, 1000, Some(100), 160);
        let k2 = fnv1a_key(p, 1001, Some(100), 160);
        let k3 = fnv1a_key(p, 1000, Some(101), 160);
        let k4 = fnv1a_key(p, 1000, Some(100), 200);
        assert_ne!(k1, k2);
        assert_ne!(k1, k3);
        assert_ne!(k1, k4);
    }

    #[test]
    fn round_trip_store_and_load_file() {
        let dir = std::env::temp_dir().join(format!("spedimage_thumb_test_{}", std::process::id()));
        let file = dir.join("test.thmb");
        let pixels = vec![42u8; 4 * 4 * 4];
        store_to_file(&file, 4, 4, &pixels).unwrap();
        let (loaded, w, h) = load_from_file(&file, 160).unwrap();
        assert_eq!((w, h), (4, 4));
        assert_eq!(loaded.as_slice(), pixels.as_slice());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
