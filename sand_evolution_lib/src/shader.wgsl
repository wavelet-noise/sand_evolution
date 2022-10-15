// Vertex shader

struct WorldSettings {
    time: f32,
    _wasm_padding0: f32,
    _wasm_padding1: f32,
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



fn hash23(p: vec2<f32>) -> vec3<f32> {
  let q = vec3<f32>(dot(p, vec2<f32>(127.1, 311.7)),
      dot(p, vec2<f32>(269.5, 183.3)),
      dot(p, vec2<f32>(419.2, 371.9)));
  return fract(sin(q) * 43758.5453);
}

fn voroNoise2(x: vec2<f32>, u: f32, v: f32) -> f32 {
  let p = floor(x);
  let f = fract(x);
  let k = 1. + 63. * pow(1. - v, 4.);
  var va: f32 = 0.;
  var wt: f32 = 0.;
  for(var j: i32 = -2; j <= 2; j = j + 1) {
    for(var i: i32 = -2; i <= 2; i = i + 1) {
      let g = vec2<f32>(f32(i), f32(j));
      let o = hash23(p + g) * vec3<f32>(u, u, 1.);
      let r = g - f + o.xy;
      let d = dot(r, r);
      let ww = pow(1. - smoothstep(0., 1.414, sqrt(d)), k);
      va = va + o.z * ww;
      wt = wt + ww;
    }
  }
  return va / wt;
}

fn rand2(n: vec2<f32>) -> f32 {
  return fract(sin(dot(n, vec2<f32>(12.9898, 4.1414))) * 43758.5453);
}

fn noise2(n: vec2<f32>) -> f32 {
  let d = vec2<f32>(0., 1.);
  let b = floor(n);
  let f = smoothstep(vec2<f32>(0.), vec2<f32>(1.), fract(n));
  return mix(mix(rand2(b), rand2(b + d.yx), f.x), mix(rand2(b + d.xy), rand2(b + d.yy), f.x), f.y);
}


@group(1) @binding(0)
var t_diffuse: texture_2d<u32>;
@group(1)@binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    //let temp: f32 = voroNoise2(in.uv * 10.0 + vec2<f32>(settings.time, settings.time), sin(settings.time), 0.0);
    //return vec4<f32>(temp,temp,temp, 1.0);

    let texel : vec4<u32> = textureLoad(t_diffuse, vec2<i32>(i32(in.uv.x * 1024.0), i32(in.uv.y * 512.0)), 0);
    let t = texel.x;

    let noisy_mixer: f32 = pow(noise2(in.uv * 400.0 + sin(settings.time)*400.0), 2.0);

    var col = vec4<f32>(0.0);

    if t == 255u
    {
      col = vec4<f32>(1.0);
    }
    else if t == 0u
    {
      col = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
    else if t == 1u
    {
      let n = noise2(in.uv * 2000.0) * 0.8;
      col = vec4<f32>(n + 0.1, n + 0.1, 0.1, 1.0);
    }
    else if t == 2u
    {
      col = vec4<f32>(0.1, 0.15, 1.0, 1.0);
    }
    else if t == 3u
    {
      col = vec4<f32>(0.5);
    }
    else if t == 4u
    {
      col = mix(
        vec4<f32>(1.0, 0.0, 0.0, 1.0),
        vec4<f32>(1.0, 1.0, 0.0, 1.0),
        noisy_mixer - 0.04
      );
    }
    else if t == 5u
    {
      col = vec4<f32>(0.5, 0.2, 0.2, 1.0);
    }
    else if t == 6u
    {
      col = vec4<f32>(0.8, 1.0, 0.5, 1.0);
    }
    else if t == 7u
    {
      col = mix(
        vec4<f32>(8.0, 0.0, 0.0, 1.0),
        vec4<f32>(8.0, 0.5, 0.0, 1.0),
        noisy_mixer
      );
    }
    else if t == 8u
    {
      col = vec4<f32>(0.05);
    }
    else
    {
      col = vec4<f32>(0.0,1.0,0.0,1.0);
    }

    return col;
}