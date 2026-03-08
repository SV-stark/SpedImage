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
    _padding: f32,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@vertex
fn vertex_main(
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>
) -> VertexOutput {
    var out: VertexOutput;
    var tex = tex_coords * vec2<f32>(uniforms.crop_w, uniforms.crop_h) 
              + vec2<f32>(uniforms.crop_x, uniforms.crop_y);
    let center = vec2<f32>(0.5, 0.5);
    let rotated_tex = rotate(tex - center, uniforms.rotation) + center;
    var pos = position;

    let image_ar = uniforms.aspect_ratio;
    let window_ar = uniforms.window_aspect_ratio;
    let ratio = image_ar / window_ar;

    if (ratio > 1.0) {
        pos.y /= ratio;
    } else {
        pos.x *= ratio;
    }

    out.position = vec4<f32>(pos, 0.0, 1.0);
    out.tex_coords = rotated_tex;
    return out;
}

fn rotate(coord: vec2<f32>, angle: f32) -> vec2<f32> {
    let s = sin(angle);
    let c = cos(angle);
    return vec2<f32>(coord.x * c - coord.y * s, coord.x * s + coord.y * c);
}

struct FragmentInput {
    @location(0) tex_coords: vec2<f32>,
};

@group(0) @binding(1)
var image_sampler: sampler;

@group(0) @binding(2)
var image_texture: texture_2d<f32>;

@fragment
fn fragment_main(input: FragmentInput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(image_texture, image_sampler, input.tex_coords);
    var color = tex_color.rgb;
    
    // Grayscale factor
    let gray = dot(color, vec3<f32>(0.299, 0.587, 0.114));

    color = color * uniforms.brightness;
    color = (color - vec3<f32>(0.5)) * uniforms.contrast + vec3<f32>(0.5);
    color = mix(vec3<f32>(gray), color, uniforms.saturation);

    if (uniforms.hdr_toning > 0.5) {
        let exposed = color * 1.6;
        color = exposed / (1.0 + exposed);
        color = color * color * (3.0 - 2.0 * color);
    }

    return vec4<f32>(color, tex_color.a);
}
"#;

pub const CROP_SHADER: &str = r#"
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
};

@vertex
fn vertex_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    // Generate a full screen quad
    var out: VertexOutput;
    let x = f32(vertex_index & 1u) * 2.0 - 1.0;
    let y = f32((vertex_index >> 1u) & 1u) * 2.0 - 1.0;
    // We invert y for Vulkan/WGPU coordinates
    out.position = vec4<f32>(x, -y, 0.0, 1.0);
    return out;
}

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // In viewport coordinates (0 to width/height)
    // Actually, in.position.xy is in pixel coordinates.
    // We pass normalized crop rect (0.0 to 1.0) but we don't know window bounds in shader easily.
    // Instead we can generate the crop rect in vertex shader or just draw the overlay regions.
    return vec4<f32>(0.0, 0.0, 0.0, 0.5); // Just a generic darken, we will use scissors!
}
"#;
