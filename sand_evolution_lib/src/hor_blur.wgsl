// Vertex shader

struct WorldSettings {
    time: f32,
    res_x: f32,
    res_y: f32,
    _wasm_padding2: f32,
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

fn getOffset(index: u32) -> f32 {
    switch (index) {
        case 0u:  { return -5.0; }
        case 1u:  { return -4.0; }
        case 2u:  { return -3.0; }
        case 3u:  { return -2.0; }
        case 4u:  { return -1.0; }
        case 5u:  { return 0.0; }
        case 6u:  { return 1.0; }
        case 7u:  { return 2.0; }
        case 8u:  { return 3.0; }
        case 9u:  { return 4.0; }
        case 10u: { return 5.0; }
        default:  { return 0.0; }
    }
}

fn getWeight(index: u32) -> f32 {
    switch (index) {
        case 0u:  { return 0.01; }
        case 1u:  { return 0.02; }
        case 2u:  { return 0.04; }
        case 3u:  { return 0.06; }
        case 4u:  { return 0.09; }
        case 5u:  { return 0.12; }
        case 6u:  { return 0.09; }
        case 7u:  { return 0.06; }
        case 8u:  { return 0.04; }
        case 9u:  { return 0.02; }
        case 10u: { return 0.01; }
        default:  { return 0.0; }
    }
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
   var uv: vec2<f32> = in.uv;
   uv.y = 1.0 - uv.y;

   var color: vec4<f32> = vec4<f32>(0.0, 0.0, 0.0, 0.0);
   let range: f32 = 1.0 / settings.res_x;

   for (var i = 0u; i < 11u; i = i + 1u) {
       let offset: f32 = getOffset(i) + 0.5;
       let weight: f32 = getWeight(i)*3.0;
       let sample_uv: vec2<f32> = vec2<f32>(uv.x + offset * range, uv.y);
       color = color + textureSample(t_diffuse, s_diffuse, sample_uv) * weight;
   }

   return color;
}