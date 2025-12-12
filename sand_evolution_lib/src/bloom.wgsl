// Vertex shader

struct WorldSettings {
    time: f32,
    res_x: f32,
    res_y: f32,
    display_mode: f32,
    global_temperature: f32,
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
    _pad3: f32,
    _pad4: f32,
    _pad5: f32,
};
@group(0) @binding(0)
var<uniform> settings: WorldSettings;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.uv = model.uv;
    out.clip_position = vec4<f32>(model.position, 1.0);
    return out;
}

// Fragment shader

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;
@group(1) @binding(2)
var t_blured: texture_2d<f32>;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    //let texel : vec4<f32> = textureLoad(t_diffuse, vec2<i32>(i32(in.uv.x * 1024.0), 512 - i32(in.uv.y * 512.0)), 0);
    var uv: vec2<f32> = in.uv;
    uv.y = 1.0 - uv.y;
    let color = textureSample(t_diffuse, s_diffuse, uv);
    let color2 = textureSample(t_blured, s_diffuse, uv);
    return color2 + color;
}