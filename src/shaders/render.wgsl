struct Uniforms {
    scale: f32,
    aspect_ratio: f32,
    camera_translation: vec2<f32>,
    window_size: vec2<f32>,
    rotation_angle: f32,
    _padding: f32,
    rotation_center: vec2<f32>,
};
@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) quad_pos: vec2<f32>,
    @location(1) particle_pos: vec2<f32>,
    @location(2) particle_vel: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    let particle_size = 0.5 / sqrt(uniforms.scale);
    let world_pos = input.particle_pos + input.quad_pos * particle_size;

    // Dragged position first
    let translated_pos = world_pos - uniforms.camera_translation;

    // second, rotation around ( rotation_center - camera_translation )
    let rotation_center_translated = uniforms.rotation_center - uniforms.camera_translation;
    let pos_relative_to_center = translated_pos - rotation_center_translated;
    let rotated_pos = rotate2d(pos_relative_to_center, uniforms.rotation_angle);
    let final_pos = rotated_pos + rotation_center_translated;

    // Lastly scale
    let camera_pos = final_pos * uniforms.scale;

    var ndc: vec2<f32>;
    ndc = camera_pos / uniforms.window_size * 2.0;

    var output: VertexOutput;
    output.position = vec4<f32>(ndc, 0.0, 1.0);

    output.color = vec4<f32>(input.particle_vel, 0.0, 1.0);
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let speed = length(input.color.xy);
    let hue = 0.3 * log(speed);

    let hsv = vec3<f32>(hue, 1.0, 1.0 - 0.0 * 0.5 * speed);
    let rgb = hsv_to_rgb(hsv);

    return vec4<f32>(rgb, 1.0);
}

fn rotate2d(pos: vec2<f32>, angle: f32) -> vec2<f32> {
    let cos_angle = cos(angle);
    let sin_angle = sin(angle);
    return vec2<f32>(
        pos.x * cos_angle - pos.y * sin_angle,
        pos.x * sin_angle + pos.y * cos_angle
    );
}

fn modulo(a: f32, b: f32) -> f32 {
    return a - b * floor(a / b);
}

fn hsv_to_rgb(hsv: vec3<f32>) -> vec3<f32> {
    let h = fract(hsv.x);
    let s = hsv.y;
    let v = hsv.z;

    let c = v * s;
    let x = c * (1.0 - abs(modulo(h * 6.0, 2.0) - 1.0));
    let m = v - c;

    if h < 1.0 / 6.0 {
        return vec3<f32>(c, x, 0.0) + vec3<f32>(m, m, m);
    } else if h < 2.0 / 6.0 {
        return vec3<f32>(x, c, 0.0) + vec3<f32>(m, m, m);
    } else if h < 3.0 / 6.0 {
        return vec3<f32>(0.0, c, x) + vec3<f32>(m, m, m);
    } else if h < 4.0 / 6.0 {
        return vec3<f32>(0.0, x, c) + vec3<f32>(m, m, m);
    } else if h < 5.0 / 6.0 {
        return vec3<f32>(x, 0.0, c) + vec3<f32>(m, m, m);
    } else {
        return vec3<f32>(c, 0.0, x) + vec3<f32>(m, m, m);
    }
}