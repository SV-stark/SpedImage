pub const SHADER: &str = r#"
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
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
        // Adjust for aspect ratio fit
        let ratio = uniforms.aspect_ratio / uniforms.window_aspect_ratio;
        if (ratio > 1.0) {
            pos.y /= ratio;
        } else {
            pos.x *= ratio;
        }

        // Apply rotation
        let angle = uniforms.rotation;
        let c = cos(angle);
        let s = sin(angle);
        let rotated_pos = vec2<f32>(
            pos.x * c - pos.y * s,
            pos.x * s + pos.y * c
        );
        pos = rotated_pos;
    }

    out.position = vec4<f32>(pos, 0.0, 1.0);
    
    // Apply crop to texture coordinates
    out.tex_coords = vec2<f32>(
        uniforms.crop_x + tex_coords.x * uniforms.crop_w,
        uniforms.crop_y + tex_coords.y * uniforms.crop_h
    );
    
    return out;
}

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let color_new = textureSample(t, s, in.tex_coords);
    let color_old = textureSample(t_prev, s, in.tex_coords);
    
    var color = mix(color_old, color_new, uniforms.transition_factor);
    
    // 1. Adjust Brightness & Contrast
    color = vec4<f32>((color.rgb - 0.5) * uniforms.contrast + 0.5 + (uniforms.brightness - 1.0), color.a);
    
    // 2. Adjust Saturation
    let gray = dot(color.rgb, vec3<f32>(0.299, 0.587, 0.114));
    color = vec4<f32>(mix(vec3<f32>(gray), color.rgb, uniforms.saturation), color.a);
    
    // 3. HDR Toning (Filmic/Reinhard)
    if (uniforms.hdr_toning > 0.5) {
        var x = color.rgb * 1.6;
        x = x / (1.0 + x);
        color = vec4<f32>(x * x * (3.0 - 2.0 * x), color.a);
    }
    
    return color;
}
"#;

pub const CROP_SHADER: &str = r#"
@vertex
fn vertex_main(@builtin(vertex_index) item_index: u32) -> @builtin(position) vec4<f32> {
    var pos = array<vec2<f32>, 4>(
        vec2<f32>(-1.0,  1.0),
        vec2<f32>( 1.0,  1.0),
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0, -1.0)
    );
    return vec4<f32>(pos[item_index], 0.0, 1.0);
}

@fragment
fn fragment_main() -> @location(0) vec4<f32> {
    return vec4<f32>(0.0, 0.0, 0.0, 0.5); // Darken for crop regions
}
"#;
