pub const SHADER: &str = r#"
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) uv: vec2<f32>,
};

struct Uniforms {
    rotation: f32,
    aspect_ratio: f32,
    window_aspect_ratio: f32,
    crop_x: f32,
    crop_y: f32,
    crop_w: f32,
    crop_h: f32,
    brightness: f32,
    contrast: f32,
    saturation: f32,
    hdr_toning: f32,
    transition_factor: f32,
    pos_offset: vec2<f32>,
    pos_scale: vec2<f32>,
    flip_horizontal: f32,
    flip_vertical: f32,
    sharpen: f32,
    clarity: f32,
    temperature: f32,
    tint: f32,
    highlights: f32,
    shadows: f32,
    split_compare: f32,
    split_position: f32,
    has_color_matrix: f32,
    _pad: f32,
    color_matrix_col0: vec4<f32>,
    color_matrix_col1: vec4<f32>,
    color_matrix_col2: vec4<f32>,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var s: sampler;
@group(0) @binding(2) var t: texture_2d<f32>;
@group(0) @binding(3) var t_prev: texture_2d<f32>;

@vertex
fn vertex_main(
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    
    var pos = position;
    
    // Scale and offset for thumbnails/UI
    pos = pos * uniforms.pos_scale + uniforms.pos_offset;
    
    // Default image rendering if not overridden by UI
    if (uniforms.pos_scale.x == 1.0 && uniforms.pos_scale.y == 1.0 && uniforms.pos_offset.x == 0.0) {
        // Apply rotation first
        let angle = uniforms.rotation;
        let c = cos(angle);
        let s = sin(angle);
        let rotated_pos = vec2<f32>(
            pos.x * c - pos.y * s,
            pos.x * s + pos.y * c
        );
        pos = rotated_pos;

        // Adjust for aspect ratio fit (using rotated aspect ratio)
        let ratio = uniforms.aspect_ratio / uniforms.window_aspect_ratio;
        if (ratio > 1.0) {
            pos.y /= ratio;
        } else {
            pos.x *= ratio;
        }
    }

    out.position = vec4<f32>(pos, 0.0, 1.0);
    out.uv = tex_coords;
    
    // Apply crop to texture coordinates (with horizontal and vertical flipping support)
    var tc = tex_coords;
    if (uniforms.flip_horizontal > 0.5) {
        tc.x = 1.0 - tc.x;
    }
    if (uniforms.flip_vertical > 0.5) {
        tc.y = 1.0 - tc.y;
    }

    out.tex_coords = vec2<f32>(
        uniforms.crop_x + tc.x * uniforms.crop_w,
        uniforms.crop_y + tc.y * uniforms.crop_h
    );
    
    return out;
}

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    if (in.tex_coords.x < 0.0 || in.tex_coords.x > 1.0 || in.tex_coords.y < 0.0 || in.tex_coords.y > 1.0) {
        return vec4<f32>(0.047, 0.055, 0.09, 1.0); // Clean dark canvas (#0c0e17) for zoomed out border
    }
    let color_new = textureSample(t, s, in.tex_coords);
    // Skip the previous-frame fetch entirely once a transition has settled;
    // this is a uniform branch, so both paths are valid uniform control flow.
    var base_color = color_new;
    if (uniforms.transition_factor < 1.0) {
        let color_old = textureSample(t_prev, s, in.tex_coords);
        base_color = mix(color_old, color_new, uniforms.transition_factor);
    }
    
    // Split screen A/B comparison divider & left side (original)
    if (uniforms.split_compare > 0.5) {
        let diff = in.uv.x - uniforms.split_position;
        if (abs(diff) < 0.0025) {
            return vec4<f32>(0.0, 0.706, 0.847, 1.0); // Cyan divider line
        }
        if (diff < 0.0) {
            return base_color; // Left side: original untouched image
        }
    }
    
    var color = base_color;
    
    // 0. Custom Color Space Correction (e.g. Adobe RGB to sRGB)
    if (uniforms.has_color_matrix > 0.5) {
        let m = mat3x3<f32>(
            uniforms.color_matrix_col0.xyz,
            uniforms.color_matrix_col1.xyz,
            uniforms.color_matrix_col2.xyz
        );
        color = vec4<f32>(m * color.rgb, color.a);
    }
    
    // 1. Sharpening (Laplacian High-Pass)
    if (uniforms.sharpen > 0.001) {
        let dims = vec2<f32>(textureDimensions(t));
        if (dims.x > 0.0 && dims.y > 0.0) {
            let texel = vec2<f32>(1.0 / dims.x, 1.0 / dims.y);
            let c_up    = textureSample(t, s, in.tex_coords + vec2<f32>(0.0, -texel.y));
            let c_down  = textureSample(t, s, in.tex_coords + vec2<f32>(0.0,  texel.y));
            let c_left  = textureSample(t, s, in.tex_coords + vec2<f32>(-texel.x, 0.0));
            let c_right = textureSample(t, s, in.tex_coords + vec2<f32>( texel.x, 0.0));
            let laplacian = (c_up.rgb + c_down.rgb + c_left.rgb + c_right.rgb) * 0.25;
            let high_pass = color.rgb - laplacian;
            color = vec4<f32>(clamp(color.rgb + high_pass * (uniforms.sharpen * 1.5), vec3<f32>(0.0), vec3<f32>(1.0)), color.a);
        }
    }
    
    // 2. White Balance (Temperature & Tint)
    if (abs(uniforms.temperature) > 0.001 || abs(uniforms.tint) > 0.001) {
        var rgb = color.rgb;
        rgb.r += uniforms.temperature * 0.22;
        rgb.g += uniforms.temperature * 0.06;
        rgb.b -= uniforms.temperature * 0.22;

        rgb.r += uniforms.tint * 0.12;
        rgb.g -= uniforms.tint * 0.22;
        rgb.b += uniforms.tint * 0.12;

        color = vec4<f32>(clamp(rgb, vec3<f32>(0.0), vec3<f32>(1.0)), color.a);
    }
    
    // 3. Tone Recovery (Shadows & Highlights)
    if (abs(uniforms.shadows) > 0.001 || abs(uniforms.highlights) > 0.001) {
        let lum = dot(color.rgb, vec3<f32>(0.299, 0.587, 0.114));
        let shadow_mask = clamp(1.0 - lum / 0.55, 0.0, 1.0);
        let highlight_mask = clamp((lum - 0.45) / 0.55, 0.0, 1.0);

        let shadow_adj = color.rgb * (uniforms.shadows * shadow_mask * 0.5);
        let highlight_adj = (vec3<f32>(1.0) - color.rgb) * (uniforms.highlights * highlight_mask * 0.5);

        color = vec4<f32>(clamp(color.rgb + shadow_adj + highlight_adj, vec3<f32>(0.0), vec3<f32>(1.0)), color.a);
    }
    
    // 4. Clarity (Midtone Contrast)
    if (abs(uniforms.clarity) > 0.001) {
        let lum = dot(color.rgb, vec3<f32>(0.299, 0.587, 0.114));
        let midtone_mask = 1.0 - abs(lum - 0.5) * 2.0;
        let clarity_adj = (color.rgb - vec3<f32>(0.5)) * (uniforms.clarity * 0.4 * max(midtone_mask, 0.0));
        color = vec4<f32>(clamp(color.rgb + clarity_adj, vec3<f32>(0.0), vec3<f32>(1.0)), color.a);
    }
    
    // 5. Adjust Brightness & Contrast
    color = vec4<f32>((color.rgb - 0.5) * uniforms.contrast + 0.5 + (uniforms.brightness - 1.0), color.a);
    
    // 6. Adjust Saturation
    let gray = dot(color.rgb, vec3<f32>(0.299, 0.587, 0.114));
    color = vec4<f32>(mix(vec3<f32>(gray), color.rgb, uniforms.saturation), color.a);
    
    // 7. HDR Toning (Filmic/Reinhard)
    if (uniforms.hdr_toning > 0.5) {
        var x = color.rgb * 1.6;
        x = x / (1.0 + x);
        color = vec4<f32>(x * x * (3.0 - 2.0 * x), color.a);
    }
    
    // 8. Transparency Handling with Subtle Checkerboard Canvas
    if (color.a < 0.999) {
        let checker_size = 16.0;
        let check_val = (floor(in.position.x / checker_size) + floor(in.position.y / checker_size)) % 2.0;
        let check_bg = select(vec3<f32>(0.14, 0.15, 0.19), vec3<f32>(0.20, 0.22, 0.27), check_val > 0.5);
        color = vec4<f32>(mix(check_bg, color.rgb, color.a), 1.0);
    }
    
    return color;
}
"#;

/// Fullscreen blit used to generate successive mipmap levels on the GPU.
pub const MIP_SHADER: &str = r#"
struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@group(0) @binding(0) var s: sampler;
@group(0) @binding(1) var t: texture_2d<f32>;

@vertex
fn vertex_main(@builtin(vertex_index) idx: u32) -> VsOut {
    var p = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>(-1.0,  1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>( 1.0,  1.0),
        vec2<f32>(-1.0,  1.0)
    );
    var out: VsOut;
    out.pos = vec4<f32>(p[idx], 0.0, 1.0);
    out.uv = vec2<f32>((p[idx].x + 1.0) * 0.5, (1.0 - p[idx].y) * 0.5);
    return out;
}

@fragment
fn fragment_main(in: VsOut) -> @location(0) vec4<f32> {
    return textureSample(t, s, in.uv);
}
"#;
