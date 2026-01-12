
struct WorldSettings {
    time: f32,
    res_x: f32,
    res_y: f32,
    display_mode: f32, // 0.0 = Normal, 1.0 = Temperature, 2.0 = Both
    global_temperature: f32,
    // Directional light direction in texel space.
    sun_dir_x: f32,
    sun_dir_y: f32,
    // Shadows.
    shadow_strength: f32,          // 0..2 (values > 1 push towards pure black)
    shadow_length_steps: f32,      // 1..64
    shadow_distance_falloff: f32,  // 0..4 (0 disables distance attenuation)
    // Background.
    bg_saturation: f32,            // 0..1
    bg_brightness: f32,            // 0.1..5
    // Padding / reserved (keep struct size a multiple of 16 bytes, in sync with Rust).
    _pad_heat0: f32,
    _pad_heat1: f32,
    _pad_heat2: f32,
    _pad_heat3: f32,
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

fn horizontalWoodPattern(uv: vec2<f32>, grain: f32, freq: f32, noiseScale: f32) -> f32 {
    let yValue = uv.y * freq;
    let wood = sin(yValue + grain * noise2(vec2<f32>(yValue, uv.x) * noiseScale));
    return wood;
}

fn permute_3_(x: vec3<f32>) -> vec3<f32> {
    return (((x * 34.) + 1.) * x) % vec3(289.);
}

fn simplex_noise_2d(v: vec2<f32>) -> f32 {
    let C = vec4(
        0.211324865405187, // (3.0 - sqrt(3.0)) / 6.0
        0.366025403784439, // 0.5 * (sqrt(3.0) - 1.0)
        -0.577350269189626, // -1.0 + 2.0 * C.x
        0.024390243902439 // 1.0 / 41.0
    );

    // first corner
    var i = floor(v + dot(v, C.yy));
    let x0 = v - i + dot(i, C.xx);

    // other corners
    var i1 = select(vec2(0., 1.), vec2(1., 0.), x0.x > x0.y);
    var x12 = x0.xyxy + C.xxzz - vec4(i1, 0., 0.);

    // permutations
    i = i % vec2(289.);

    let p = permute_3_(permute_3_(i.y + vec3(0., i1.y, 1.)) + i.x + vec3(0., i1.x, 1.));
    var m = max(0.5 - vec3(dot(x0, x0), dot(x12.xy, x12.xy), dot(x12.zw, x12.zw)), vec3(0.));
    m *= m;
    m *= m;

    // gradients: 41 points uniformly over a line, mapped onto a diamond
    // the ring size, 17*17 = 289, is close to a multiple of 41 (41*7 = 287)
    let x = 2. * fract(p * C.www) - 1.;
    let h = abs(x) - 0.5;
    let ox = floor(x + 0.5);
    let a0 = x - ox;

    // normalize gradients implicitly by scaling m
    // approximation of: m *= inversesqrt(a0 * a0 + h * h);
    m = m * (1.79284291400159 - 0.85373472095314 * (a0 * a0 + h * h));

    // compute final noise value at P
    let g = vec3(a0.x * x0.x + h.x * x0.y, a0.yz * x12.xz + h.yz * x12.yw);
    return 130. * dot(m, g);
}

fn simplex_noise_2d_seeded(v: vec2<f32>, seed: f32) -> f32 {
    let C = vec4(
        0.211324865405187, // (3.0 - sqrt(3.0)) / 6.0
        0.366025403784439, // 0.5 * (sqrt(3.0) - 1.0)
        -0.577350269189626, // -1.0 + 2.0 * C.x
        0.024390243902439 // 1.0 / 41.0
    );

    // first corner
    var i = floor(v + dot(v, C.yy));
    let x0 = v - i + dot(i, C.xx);

    // other corners
    var i1 = select(vec2(0., 1.), vec2(1., 0.), x0.x > x0.y);
    var x12 = x0.xyxy + C.xxzz - vec4(i1, 0., 0.);

    // permutations
    i = i % vec2(289.);

    var p = permute_3_(permute_3_(i.y + vec3(0., i1.y, 1.)) + i.x + vec3(0., i1.x, 1.));
    p = permute_3_(p + vec3(seed));
    var m = max(0.5 - vec3(dot(x0, x0), dot(x12.xy, x12.xy), dot(x12.zw, x12.zw)), vec3(0.));
    m *= m;
    m *= m;

    // gradients: 41 points uniformly over a line, mapped onto a diamond
    // the ring size, 17*17 = 289, is close to a multiple of 41 (41*7 = 287)
    let x = 2. * fract(p * C.www) - 1.;
    let h = abs(x) - 0.5;
    let ox = floor(x + 0.5);
    let a0 = x - ox;

    // normalize gradients implicitly by scaling m
    // approximation of: m *= inversesqrt(a0 * a0 + h * h);
    m = m * (1.79284291400159 - 0.85373472095314 * (a0 * a0 + h * h));

    // compute final noise value at P
    let g = vec3(a0.x * x0.x + h.x * x0.y, a0.yz * x12.xz + h.yz * x12.yw);
    return 130. * dot(m, g);
}

fn permute_4_(x: vec4<f32>) -> vec4<f32> {
    return ((x * 34. + 1.) * x) % vec4<f32>(289.);
}

fn taylor_inv_sqrt_4_(r: vec4<f32>) -> vec4<f32> {
    return 1.79284291400159 - 0.85373472095314 * r;
}

fn simplex_noise_3d(v: vec3<f32>) -> f32 {
    let C = vec2(1. / 6., 1. / 3.);
    let D = vec4(0., 0.5, 1., 2.);

    // first corner
    var i = floor(v + dot(v, C.yyy));
    let x0 = v - i + dot(i, C.xxx);

    // other corners
    let g = step(x0.yzx, x0.xyz);
    let l = 1. - g;
    let i1 = min(g.xyz, l.zxy);
    let i2 = max(g.xyz, l.zxy);

    // x0 = x0 - 0. + 0. * C
    let x1 = x0 - i1 + 1. * C.xxx;
    let x2 = x0 - i2 + 2. * C.xxx;
    let x3 = x0 - 1. + 3. * C.xxx;

    // permutations
    i = i % vec3(289.);
    let p = permute_4_(permute_4_(permute_4_(
        i.z + vec4(0., i1.z, i2.z, 1.)) +
        i.y + vec4(0., i1.y, i2.y, 1.)) +
        i.x + vec4(0., i1.x, i2.x, 1.)
    );

    // gradients (NxN points uniformly over a square, mapped onto an octahedron)
    let n_ = 1. / 7.; // N=7
    let ns = n_ * D.wyz - D.xzx;

    let j = p - 49. * floor(p * ns.z * ns.z); // mod(p, N*N)

    let x_ = floor(j * ns.z);
    let y_ = floor(j - 7. * x_); // mod(j, N)

    let x = x_ * ns.x + ns.yyyy;
    let y = y_ * ns.x + ns.yyyy;
    let h = 1. - abs(x) - abs(y);

    let b0 = vec4(x.xy, y.xy);
    let b1 = vec4(x.zw, y.zw);

    let s0 = floor(b0) * 2. + 1.;
    let s1 = floor(b1) * 2. + 1.;
    let sh = -step(h, vec4(0.));

    let a0 = b0.xzyw + s0.xzyw * sh.xxyy;
    let a1 = b1.xzyw + s1.xzyw * sh.zzww;

    var p0 = vec3(a0.xy, h.x);
    var p1 = vec3(a0.zw, h.y);
    var p2 = vec3(a1.xy, h.z);
    var p3 = vec3(a1.zw, h.w);

    // normalize gradients
    let norm = taylor_inv_sqrt_4_(vec4(dot(p0, p0), dot(p1, p1), dot(p2, p2), dot(p3, p3)));
    p0 = p0 * norm.x;
    p1 = p1 * norm.y;
    p2 = p2 * norm.z;
    p3 = p3 * norm.w;

    // mix final noise value
    var m = 0.6 - vec4(dot(x0, x0), dot(x1, x1), dot(x2, x2), dot(x3, x3));
    m = max(m, vec4(0.));
    m *= m;
    return 42. * dot(m * m, vec4(dot(p0, x0), dot(p1, x1), dot(p2, x2), dot(p3, x3)));
}


fn fbm_simplex_2d(pos: vec2<f32>, octaves: i32, lacunarity: f32, gain: f32) -> f32 {
    var sum = 0.;
    var amplitude = 1.;
    var frequency = 1.;

    for (var i = 0; i < octaves; i+= 1) {
        sum += simplex_noise_2d(pos * frequency) * amplitude;
        amplitude *= gain;
        frequency *= lacunarity;
    }

    return sum;
}

fn fbm_simplex_2d_seeded(pos: vec2<f32>, octaves: i32, lacunarity: f32, gain: f32, seed: f32) -> f32 {
    var sum = 0.;
    var amplitude = 1.;
    var frequency = 1.;

    for (var i = 0; i < octaves; i+= 1) {
        sum += simplex_noise_2d_seeded(pos * frequency, seed) * amplitude;
        amplitude *= gain;
        frequency *= lacunarity;
    }

    return sum;
}

fn fbm_simplex_3d(pos: vec3<f32>, octaves: i32, lacunarity: f32, gain: f32) -> f32 {
    var sum = 0.;
    var amplitude = 1.;
    var frequency = 1.;

    for (var i = 0; i < octaves; i+= 1) {
        sum += simplex_noise_3d(pos * frequency) * amplitude;
        amplitude *= gain;
        frequency *= lacunarity;
    }

    return sum;
}

struct FragmentOutput {
    @location(0) albedo: vec4<f32>,
    @location(1) bloom: vec4<f32>
}

@group(1) @binding(0)
var t_diffuse: texture_2d<u32>;
@group(1)@binding(1)
var s_diffuse: sampler;

@group(2) @binding(0)
var t_temperature: texture_2d<f32>;
@group(2) @binding(1)
var s_temperature: sampler;

@group(3) @binding(0)
var t_shadow_props: texture_2d<f32>;
@group(3) @binding(1)
var s_shadow_props: sampler;

fn shadow_props(id: u32) -> vec4<f32> {
    // Texture is 256x1 (RGBA8Unorm). Channels are normalized to [0..1].
    // - rgb: shadow multiplier color (1.0 = no darkening)
    // - a:   shadow opacity/strength (0.0 = does not affect)
    return textureLoad(t_shadow_props, vec2<i32>(i32(id), 0), 0);
}

fn sample_temperature_bilinear(uv: vec2<f32>, dims_i: vec2<i32>, dims_f: vec2<f32>) -> f32 {
    // Map uv [0..1] to texel space, aligned to texel centers.
    let pos = uv * dims_f - vec2<f32>(0.5, 0.5);
    let base = vec2<i32>(i32(floor(pos.x)), i32(floor(pos.y)));
    let fx = fract(pos.x);
    let fy = fract(pos.y);

    let max_xy = dims_i - vec2<i32>(1, 1);
    let p00 = clamp(base, vec2<i32>(0, 0), max_xy);
    let p10 = clamp(base + vec2<i32>(1, 0), vec2<i32>(0, 0), max_xy);
    let p01 = clamp(base + vec2<i32>(0, 1), vec2<i32>(0, 0), max_xy);
    let p11 = clamp(base + vec2<i32>(1, 1), vec2<i32>(0, 0), max_xy);

    let t00 = textureLoad(t_temperature, p00, 0).r;
    let t10 = textureLoad(t_temperature, p10, 0).r;
    let t01 = textureLoad(t_temperature, p01, 0).r;
    let t11 = textureLoad(t_temperature, p11, 0).r;

    let a = mix(t00, t10, fx);
    let b = mix(t01, t11, fx);
    return mix(a, b, fy);
}

fn compute_wall_shadow(cell_xy: vec2<i32>) -> vec4<f32> {
    // Directional light direction in texel space (x,y), provided via padding fields.
    // If not set, fall back to a sensible default.
    var dir = vec2<f32>(settings.sun_dir_x, settings.sun_dir_y);
    if dot(dir, dir) < 1e-6 {
        // Almost strictly down.
        dir = vec2<f32>(0.03, -1.0);
    }
    dir = normalize(dir);

    let inv = 1.0 / max(abs(dir.x), abs(dir.y));
    let step = dir * inv;

    let MAX_STEPS: i32 = 64;
    let max_steps: i32 = clamp(i32(round(settings.shadow_length_steps)), 1, MAX_STEPS);

    let perp = vec2<f32>(-dir.y, dir.x);
    let samples: i32 = 5;
    let half_samples = f32(samples - 1) * 0.5;
    var strength_sum: f32 = 0.0;
    var mult_rgb_sum: vec3<f32> = vec3<f32>(0.0);

    let jitter = (noise2(vec2<f32>(f32(cell_xy.x), f32(cell_xy.y)) * 0.17) - 0.5) * 0.25;

    for (var s: i32 = 0; s < samples; s = s + 1) {
        let ofs = (f32(s) - half_samples) * 0.35 + jitter; // sub-texel width
        var p = vec2<f32>(f32(cell_xy.x) + 0.5, f32(cell_xy.y) + 0.5) + perp * ofs;

        var hit_strength: f32 = 0.0;
        var hit_mult_rgb: vec3<f32> = vec3<f32>(1.0);
        for (var i: i32 = 1; i <= MAX_STEPS; i = i + 1) {
            if (i > max_steps) { break; }
            p -= step;
            let pi = vec2<i32>(i32(floor(p.x)), i32(floor(p.y)));

            if (pi.x < 0 || pi.y < 0 || pi.x >= i32(settings.res_x) || pi.y >= i32(settings.res_y)) {
                break;
            }

            // Shadow caster check is controlled by per-cell shadow props (alpha=0 => does not occlude).
            let tt = textureLoad(t_diffuse, pi, 0).x;
            if (tt != 0u) {
                let sp = shadow_props(tt);
                if (sp.a > 0.001) {
                    // Fade with distance: closer occluder => stronger.
                    let dist_base = max(0.0, 1.0 - (f32(i) / f32(max_steps)));
                    let falloff = max(settings.shadow_distance_falloff, 0.0);
                    let dist_k = select(pow(dist_base, falloff), 1.0, falloff < 1e-3);
                    hit_strength = dist_k * sp.a;
                    hit_mult_rgb = sp.rgb;
                    break;
                }
            }
        }

        strength_sum += hit_strength;
        mult_rgb_sum += hit_mult_rgb * hit_strength;
    }

    // Don't average by samples: more rays hitting => stronger shadow.
    let strength = clamp(strength_sum, 0.0, 1.0);
    var mult_rgb = vec3<f32>(1.0);
    if (strength_sum > 1e-6) {
        mult_rgb = clamp(mult_rgb_sum / strength_sum, vec3<f32>(0.0), vec3<f32>(1.0));
    }
    return vec4<f32>(mult_rgb, strength);
}

fn wall_background_albedo(uv: vec2<f32>, cell_xy: vec2<i32>) -> vec3<f32> {
    // Procedural "test room wall" background:
    // brick pattern + mortar + subtle plaster/grime, stable in cell-space.
    let p = vec2<f32>(f32(cell_xy.x), f32(cell_xy.y));

    // Brick dimensions in cells (tweak for scale).
    let brick_size = vec2<f32>(16.0, 8.0);
    var b = p / brick_size;

    // Offset every other row by half a brick.
    let row_i: i32 = i32(floor(b.y));
    let row_odd: bool = (row_i & 1) == 1;
    let x_off = select(0.0, 0.5, row_odd);
    b = vec2<f32>(b.x + x_off, b.y);

    let brick_id = floor(b);
    let f = fract(b);

    // Mortar thickness (fraction of brick tile).
    // Add slight edge roughness so bricks aren't perfect rectangles.
    let mortar = 0.075;
    let edge0 = min(min(f.x, 1.0 - f.x), min(f.y, 1.0 - f.y));
    // Only affect pixels close to mortar lines; keep brick interiors stable.
    let rough_n = clamp(fbm_simplex_2d(p * 0.35 + brick_id * 1.7, 2, 2.0, 0.5) * 0.5 + 0.5, 0.0, 1.0);
    let rough = (rough_n - 0.5) * 0.028; // +/- ~0.014
    let edge = edge0 + rough * smoothstep(0.18, 0.0, edge0);
    let mortar_mask = smoothstep(mortar, 0.0, edge); // 1 at edges (mortar), 0 inside brick

    // Brick color / wear variation per-brick (stable).
    let hh = hash23(brick_id);
    let h = hh.x;
    let h2 = hh.y;
    let base_a = vec3<f32>(0.48, 0.27, 0.22);
    let base_b = vec3<f32>(0.66, 0.41, 0.32);
    var brick = mix(base_a, base_b, h);

    // Add gentle within-brick variation + pores.
    let fine = fbm_simplex_2d(p * 0.08, 3, 2.0, 0.5) * 0.5 + 0.5;
    let pores = voroNoise2(p * 0.12, 0.65, 0.8);
    brick *= 0.88 + 0.18 * fine;
    brick *= 0.92 + 0.10 * pores;

    // Make some bricks noticeably darker (stable per brick_id).
    // ~40% bricks will be darkened, with varying strength.
    let dark_sel = smoothstep(0.60, 0.92, h2);
    brick *= 1.0 - 0.42 * dark_sel;

    // Some bricks are more worn. Use a per-brick mask so only a subset gets affected.
    let worn_brick = smoothstep(0.55, 0.85, h);
    // Wear is stronger near edges + patchy in the middle.
    let edge_wear = smoothstep(0.55, 0.05, edge); // 1 near edges
    let wear_patch = clamp(fbm_simplex_2d(p * 0.16 + brick_id * 7.3, 3, 2.1, 0.55) * 0.5 + 0.5, 0.0, 1.0);
    let wear_k = worn_brick * clamp(0.55 * edge_wear + 0.45 * smoothstep(0.55, 0.92, wear_patch), 0.0, 1.0);
    // Desaturate + lighten slightly (dust/plaster) on worn areas.
    let brick_luma = dot(brick, vec3<f32>(0.299, 0.587, 0.114));
    let dusty = mix(brick, vec3<f32>(brick_luma), 0.55);
    brick = mix(brick, dusty * vec3<f32>(1.05, 1.05, 1.06), wear_k * 0.85);
    // Small chips: a few bright specks on worn bricks.
    let chips = smoothstep(0.72, 0.93, voroNoise2(p * 0.28 + brick_id * 3.1, 0.55, 0.9));
    brick = mix(brick, brick * vec3<f32>(1.14, 1.14, 1.14), chips * wear_k * 0.28);

    var grout = vec3<f32>(0.12, 0.12, 0.125);
    let grout_n = fbm_simplex_2d(p * 0.16 + vec2<f32>(13.1, 7.7), 2, 2.2, 0.55) * 0.5 + 0.5;
    grout *= 0.92 + 0.08 * grout_n;

    // Subtle stains and plaster overlay.
    let stains = clamp(fbm_simplex_2d(p * 0.02 + vec2<f32>(91.0, 17.0), 4, 1.9, 0.55) * 0.5 + 0.5, 0.0, 1.0);
    let stain_mask = smoothstep(0.55, 0.90, stains) * 0.25;
    brick = mix(brick, brick * vec3<f32>(0.82, 0.86, 0.90), stain_mask);
    // Keep grout neutral; only slightly lighten on stains.
    grout = mix(grout, grout * vec3<f32>(0.92, 0.92, 0.93), stain_mask * 0.6);

    // Lighting: mild vignette + top lift so it reads as a "room".
    let v = 1.0 - smoothstep(0.62, 1.05, distance(uv, vec2<f32>(0.5, 0.5)));
    let top = 0.92 + 0.10 * (1.0 - uv.y);
    let light = (0.78 + 0.22 * v) * top;

    var out_rgb = mix(brick, grout, mortar_mask) * light;

    // User-controlled brightness for background only.
    let bg_brightness = clamp(settings.bg_brightness, 0.0, 5.0);
    out_rgb *= bg_brightness;

    // User-controlled saturation for background only.
    let sat = clamp(settings.bg_saturation, 0.0, 1.0);
    let luma = dot(out_rgb, vec3<f32>(0.299, 0.587, 0.114));
    out_rgb = mix(vec3<f32>(luma), out_rgb, sat);

    // Allow values > 1.0 (brightness) but keep them bounded.
    return clamp(out_rgb, vec3<f32>(0.0), vec3<f32>(8.0));
}

fn wall_brick_mortar_mask_simple(cell_xy: vec2<i32>) -> f32 {
    let p = vec2<f32>(f32(cell_xy.x), f32(cell_xy.y));

    let brick_size = vec2<f32>(16.0, 8.0);
    var b = p / brick_size;

    // Offset every other row by half a brick.
    let row_i: i32 = i32(floor(b.y));
    let row_odd: bool = (row_i & 1) == 1;
    let x_off = select(0.0, 0.5, row_odd);
    b = vec2<f32>(b.x + x_off, b.y);

    let f = fract(b);
    let mortar = 0.075;
    let edge0 = min(min(f.x, 1.0 - f.x), min(f.y, 1.0 - f.y));
    return smoothstep(mortar, 0.0, edge0); // 1 near seams, 0 inside brick
}

fn wall_brick_pseudo_normal(cell_xy: vec2<i32>) -> vec3<f32> {
    let mx1 = wall_brick_mortar_mask_simple(cell_xy + vec2<i32>( 1,  0));
    let mx0 = wall_brick_mortar_mask_simple(cell_xy + vec2<i32>(-1,  0));
    let my1 = wall_brick_mortar_mask_simple(cell_xy + vec2<i32>( 0,  1));
    let my0 = wall_brick_mortar_mask_simple(cell_xy + vec2<i32>( 0, -1));

    // Central difference in "cell units".
    let dx = (mx1 - mx0) * 0.5;
    let dy = (my1 - my0) * 0.5;

    // Z=1 gives a "flat surface" baseline; edges tilt the normal.
    return normalize(vec3<f32>(-dx, -dy, 1.0));
}

fn is_translucent_fluid_or_gas(t: u32) -> bool {
    return
        t == 2u  || // water
        t == 3u  || // steam
        t == 9u  || // acid
        t == 10u || // gas
        t == 11u || // burning gas (emissive overlay)
        t == 83u || // delute acid
        t == 84u || // salty water
        t == 85u || // base water
        t == 17u || // liquid gas
        t == 21u;   // smoke
}


fn fire_color_ramp(heat01: f32) -> vec3<f32> {
    let t = clamp(heat01, 0.0, 1.0);
    let c0 = vec3<f32>(0.02, 0.00, 0.00); // almost black
    let c1 = vec3<f32>(0.95, 0.08, 0.01); // deep red
    let c2 = vec3<f32>(1.00, 0.42, 0.05); // orange
    let c3 = vec3<f32>(1.00, 0.92, 0.22); // yellow
    let c4 = vec3<f32>(1.00, 1.00, 1.00); // white-hot

    var c = mix(c0, c1, smoothstep(0.00, 0.18, t));
    c = mix(c, c2, smoothstep(0.18, 0.50, t));
    c = mix(c, c3, smoothstep(0.50, 0.80, t));
    c = mix(c, c4, smoothstep(0.80, 1.00, t));
    return c;
}

fn brick_heat_color_ramp(heat01: f32) -> vec3<f32> {
    let t = clamp(heat01, 0.0, 1.0);
    let c0 = vec3<f32>(0.0, 0.0, 0.0);
    let c1 = vec3<f32>(0.70, 0.03, 0.00); // dull red (heated brick)
    let c2 = vec3<f32>(1.55, 0.18, 0.02); // bright red-orange
    let c3 = vec3<f32>(2.35, 0.48, 0.06); // very hot brick (still not yellow/white)

    // Keep most of the range in reds; only at the very top it shifts orange.
    var c = mix(c0, c1, smoothstep(0.00, 0.28, t));
    c = mix(c, c2, smoothstep(0.28, 0.78, t));
    c = mix(c, c3, smoothstep(0.78, 1.00, t));
    return c;
}

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    let uv = in.uv;

    // Temperature texture is lower-res (res/4).
    // In Temperature-only AND Both modes we keep the "cell-accurate" point sampling
    // (blocky on purpose) so it matches the underlying temp map data.
    // In Normal we smooth it with manual bilinear interpolation
    // (works even if the texture is not filterable).
    let temp_dims_i = vec2<i32>(
        max(1, i32(settings.res_x / 4.0)),
        max(1, i32(settings.res_y / 4.0)),
    );
    let temp_dims_f = vec2<f32>(f32(temp_dims_i.x), f32(temp_dims_i.y));

    let tex_coord_point = vec2<i32>(
        i32(uv.x * settings.res_x / 4.0),
        i32(uv.y * settings.res_y / 4.0),
    );
    let temp_point = textureLoad(t_temperature, tex_coord_point, 0).r;
    let temp_smooth = sample_temperature_bilinear(uv, temp_dims_i, temp_dims_f);

    let is_temp_point_sampling = (settings.display_mode > 0.5);
    let temp_value =
        (select(temp_smooth, temp_point, is_temp_point_sampling)) + settings.global_temperature;

    // Extra smoothing for heat visualization in Normal render (more "blurry heat").
    // Temperature grid is already low-res (res/4), so a few taps are cheap.
    let texel_uv = vec2<f32>(1.0, 1.0) / temp_dims_f;
    let t0 = temp_smooth;
    let t1 = sample_temperature_bilinear(uv + vec2<f32>( texel_uv.x, 0.0), temp_dims_i, temp_dims_f);
    let t2 = sample_temperature_bilinear(uv + vec2<f32>(-texel_uv.x, 0.0), temp_dims_i, temp_dims_f);
    let t3 = sample_temperature_bilinear(uv + vec2<f32>(0.0,  texel_uv.y), temp_dims_i, temp_dims_f);
    let t4 = sample_temperature_bilinear(uv + vec2<f32>(0.0, -texel_uv.y), temp_dims_i, temp_dims_f);
    let temp_vis = (t0 + t1 + t2 + t3 + t4) * 0.2 + settings.global_temperature;
    
    var temp_col: vec4<f32>;
    // Temperature visualization ramp (smoothly blended across 0° and 21°).
    // Goal: keep transitions readable, but avoid harsh "stripes"/kinks at branch boundaries.
    {
        // More subdued (readable) ramp:
        // keep the smooth blending across 0° and 21°, but avoid HDR-brightness and neon greens.

        // --- Cold (<=0): continuous at 0° (matches cool start)
        let cold0 = vec3<f32>(0.0, 0.15, 0.25); // 0°
        let cold1 = vec3<f32>(0.0, 0.35, 1.0);  // deep cold blue
        let cold_raw = clamp((-temp_value) / 140.0, 0.0, 1.0);
        let cold_t = pow(smoothstep(0.0, 1.0, cold_raw), 1.25);
        let cold_rgb = mix(cold0, cold1, cold_t);

        // --- Cool (0..21): keep it close to the old palette (blue-green -> cyan)
        let cool_raw = clamp(temp_value / 21.0, 0.0, 1.0);
        let cool_t = pow(smoothstep(0.0, 1.0, cool_raw), 1.10);
        let cool_rgb = mix(
            vec3<f32>(0.0, 0.15, 0.25),  // 0°
            vec3<f32>(0.0, 0.45, 0.65),  // 21°
            cool_t,
        );

        // --- Hot (>=21): keep the original "physics-ish" warm ramp (readable, not too bright)
        // Increased max temp to 5000 to allow explosive temperatures
        let hot = clamp((temp_value - 21.0) / (5000.0 - 21.0), 0.0, 1.0);
        let red   = clamp(0.4 + 0.6 * pow(hot, 0.35), 0.0, 1.0);
        let green = clamp(0.05 + 0.95 * pow(hot, 1.8), 0.0, 1.0);
        let blue  = clamp(0.0 + 0.55 * pow(hot, 3.0), 0.0, 1.0);
        let hot_rgb = vec3<f32>(red, green, blue) * hot;

        // Blend widths (degrees): boundary stays visible, but without a hard stripe.
        let w0: f32 = 4.0;  // around 0°
        let w1: f32 = 8.0;  // around 21°

        let k0 = smoothstep(-w0, w0, temp_value);
        let cc_rgb = mix(cold_rgb, cool_rgb, k0);

        let k1 = smoothstep(21.0 - w1, 21.0 + w1, temp_value);
        let rgb = mix(cc_rgb, hot_rgb, k1);

        temp_col = vec4<f32>(rgb, 1.0);
    }
    
    // Check if we're in temperature-only mode
    if settings.display_mode > 1.5 {
        // Both mode - render cells first, then overlay temperature
        // Continue to normal rendering code below
    } else if settings.display_mode > 0.5 {
        // Temperature map mode only
        var out: FragmentOutput;
        out.albedo = temp_col;
        out.bloom = vec4<f32>(0.0, 0.0, 0.0, 1.0);
        return out;
    }

    // Normal mode or Both mode - render cells
    let grain = 0.9;
    let freq = settings.res_y * 10.0;
    let noiseScale = 1.0;

    let woodColor = horizontalWoodPattern(uv, grain, freq, noiseScale);

    let texel : vec4<u32> = textureLoad(t_diffuse, vec2<i32>(i32(in.uv.x * settings.res_x), i32(in.uv.y * settings.res_y)), 0);
    let t = texel.x;
    let cell_xy = vec2<i32>(i32(in.uv.x * settings.res_x), i32(in.uv.y * settings.res_y));

    // Compute procedural wall once; reuse it for all translucent fluids/gases.
    let wall_bg = wall_background_albedo(uv, cell_xy);
    var wall_bg_shadowed = wall_bg;
    let shadow_strength = max(settings.shadow_strength, 0.0);
    let need_bg_shadow = (t == 0u) || is_translucent_fluid_or_gas(t);
    if (need_bg_shadow && shadow_strength > 0.0) {
        let shadow = compute_wall_shadow(cell_xy);
        let extra_black = clamp(shadow_strength - 1.0, 0.0, 1.0);
        var dark_rgb = wall_bg_shadowed * shadow.rgb;
        dark_rgb = mix(dark_rgb, vec3<f32>(0.0, 0.0, 0.0), extra_black);
        let k = clamp(shadow.a * shadow_strength, 0.0, 1.0);
        wall_bg_shadowed = mix(wall_bg_shadowed, dark_rgb, k);
    }

    let noisy_mixer: f32 = pow(noise2(in.uv * 800.0 + settings.time*400.0), 2.0);

    let noise_pixel = noise2(in.uv * vec2<f32>(settings.res_x, settings.res_y)*2.0);
    let sprite_pixel = noise2(floor(in.uv * vec2<f32>(settings.res_x, settings.res_y))) * noise2(floor(in.uv * vec2<f32>(settings.res_x / 10.0, settings.res_y)));

    let tdnoise = fbm_simplex_3d(vec3<f32>(uv * vec2<f32>(settings.res_x, settings.res_y), settings.time / 5.0), 4, 0.9, 0.1);
    let tdnoise_fast = fbm_simplex_3d(vec3<f32>(uv * vec2<f32>(settings.res_x, settings.res_y), settings.time), 4, 0.9, 0.1);
    let tdnoise_faster = fbm_simplex_3d(vec3<f32>(uv * vec2<f32>(settings.res_x, settings.res_y) * 0.25, settings.time* 2.0), 6, 0.9, 0.1);

    var col = vec4<f32>(0.0);

    if t == 255u
    {
      col = vec4<f32>(1.0)*((noise_pixel+1.0)/3.0);
    }
    else if t == 0u
    {
      // Background: procedural "test room wall" instead of flat gray.
      col = vec4<f32>(wall_bg_shadowed, 1.0);
    }
    else if t == 1u
    {
      col = vec4<f32>(noise_pixel * 0.8 + 0.1, noise_pixel * 0.8 + 0.1, 0.1, 1.0);
    }
    else if t == 18u // earth
    {
      // Brown soil with subtle noise variation
      let base = vec3<f32>(0.47, 0.28, 0.14);
      let n = clamp((noise_pixel + 1.0) * 0.5, 0.0, 1.0);
      let speck = clamp((sprite_pixel + 1.0) * 0.5, 0.0, 1.0);
      let rgb = base * (0.85 + 0.30 * n) + vec3<f32>(0.03, 0.02, 0.01) * speck;
      col = vec4<f32>(rgb, 1.0);
    }
    else if t == 19u // gravel
    {
      // Gravel: gray-ish stones with speckles
      let base = vec3<f32>(0.55, 0.55, 0.60);
      let n = clamp((noise_pixel + 1.0) * 0.5, 0.0, 1.0);
      let speck = clamp((sprite_pixel + 1.0) * 0.5, 0.0, 1.0);
      let rgb = mix(base * (0.85 + 0.25 * n), vec3<f32>(0.28, 0.26, 0.24), speck * 0.35);
      col = vec4<f32>(rgb, 1.0);
    }
    else if t == 2u // water
    {
      var water = mix(vec4<f32>(0.1, 0.15, 1.0, 2.0) * 0.5, vec4<f32>(0.1, 0.15, 1.1, 1.0), tdnoise);
      if (water.b > 1.0) {
        water = mix(vec4<f32>(0.8, 0.8, 1.1, 1.0), vec4<f32>(0.3, 0.3, 0.8, 1.0), noise_pixel);
      }
      // Transparency strength used in the in-shader composite pass below.
      let a = clamp(0.12 + 0.10 * (tdnoise * 0.5 + 0.5), 0.10, 0.25);
      col = vec4<f32>(water.rgb, a);
    }
    else if t == 3u // steam
    {
      let k = (tdnoise_fast + 2.0) / 3.0;
      let a = clamp(0.10 + 0.18 * k, 0.10, 0.32);
      col = vec4<f32>(vec3<f32>(0.5) * k, a);
    }
    else if t == 21u // smoke
    {
      let k = clamp((tdnoise_fast + 2.0) / 3.0, 0.0, 1.0);
      let a = clamp(0.15 + 0.10 * k, 0.6, 0.99);
      let smoke_color = vec3<f32>(0.02, 0.02, 0.02) * (0.9 + 0.1 * k);
      col = vec4<f32>(0.0,0.0,0.0, 1.0);//test
    }
    else if t == 4u // fire
    {
      // Improved fire shading with separated temperature, flame, and incandescence

let n_fast   = clamp(tdnoise_fast   * 0.5 + 0.5, 0.0, 1.0);
let n_faster = clamp(tdnoise_faster * 0.5 + 0.5, 0.0, 1.0);

var heat01 = clamp((temp_value - 220.0) / (5000.0 - 220.0), 0.0, 1.0);
heat01 = clamp(
  heat01
  + (n_faster - 0.5) * 0.16
  + (noise_pixel - 0.5) * 0.04,
  0.0, 1.0
);

let grid_uv = uv * vec2<f32>(settings.res_x, settings.res_y);
let cell_uv = fract(grid_uv);              // [0..1) inside the cell
let cell_pos = vec2<f32>(f32(cell_xy.x), f32(cell_xy.y));

let flow = vec2<f32>(0.0, -settings.time * (2.0 + 3.0 * heat01));
let flame_fbm = fbm_simplex_3d(
  vec3<f32>((cell_pos + cell_uv) * 0.55 + flow, settings.time * 1.25),
  5, 2.0, 0.5
);
let f01 = clamp(flame_fbm * 0.5 + 0.5, 0.0, 1.0);

let sway_n = simplex_noise_3d(vec3<f32>((cell_pos + cell_uv) * 0.18, settings.time * 0.9));
let sway = clamp(sway_n, -1.0, 1.0) * 0.22;

let x = (cell_uv.x - 0.5) + sway;
let y = cell_uv.y;              // y grows down
let v = clamp(y, 0.0, 1.0);     // 0 = top, 1 = base
let base = 1.0 - v;             // 1 = base, 0 = top

let width = mix(0.50, 0.14, v);
let edge = 1.0 - smoothstep(width - 0.10, width, abs(x));

let ridge = 1.0 - abs(2.0 * f01 - 1.0);
let tongues = pow(clamp(ridge, 0.0, 1.0), 1.8);

let flame_mask = clamp(
  edge *
  (0.45 + 0.75 * tongues) *
  (0.35 + 0.65 * base),
  0.0, 1.0
);

let flame01 = clamp(flame_mask * (0.6 + 0.6 * heat01), 0.0, 1.0);

let incand01 = pow(heat01, 1.25);

let color_t = clamp(max(incand01, flame01), 0.0, 1.0);
let rgb = fire_color_ramp(color_t);

let flicker = clamp(0.80 + 0.45 * n_fast, 0.70, 1.30);
let intensity =
  (3.0 + 10.0 * pow(incand01, 1.4)) *
  mix(0.7, 1.8, flame01) *
  flicker;

let spark = step(0.985, f01) * step(0.7, base) * heat01;
let spark_rgb = vec3<f32>(1.0, 0.8, 0.4) * spark * 6.0;

col = vec4<f32>(rgb * intensity + spark_rgb, 1.0);

    }
    else if t == 50u // powder (slow fire)
    {
      // Simplified powder shader
      let n_fast = clamp(tdnoise_fast * 0.5 + 0.5, 0.0, 1.0);
      
      // Increased max temp to 5000 to allow explosive temperatures from gunpowder
      var heat01 = clamp((temp_value - 180.0) / (5000.0 - 180.0), 0.0, 1.0);
      heat01 = clamp(heat01 + (noise_pixel - 0.5) * 0.05, 0.0, 1.0);
      // Removed 0.85 cap to allow full brightness at high temperatures
      
      // Base color for powder (dark gray-brown)
      let base_color = vec3<f32>(0.2, 0.15, 0.1);
      
      // Glow only when hot
      let glow_rgb = fire_color_ramp(heat01);
      let flicker = clamp(0.90 + 0.20 * n_fast, 0.80, 1.20);
      let glow_intensity = mix(0.0, 4.0, heat01) * flicker;
      
      col = vec4<f32>(base_color + glow_rgb * glow_intensity, 1.0);
    }
    else if t == 5u // wood
    {
      col = mix(vec4<f32>(0.5, 0.2, 0.2, 1.0) * 1.5, vec4<f32>(0.5, 0.2, 0.2, 1.0) * 0.5, woodColor) * 0.5 * ((noise_pixel+1.0)/3.0);
    }
    else if t == 55u // ice
    {
        col = mix(vec4<f32>(0.3, 0.6, 1.0, 2.0) * 0.5, vec4<f32>(0.3, 0.6, 1.2, 1.0), tdnoise);
        if (col.b > 1.0) {
          col = mix(vec4<f32>(0.8, 0.8, 1.1,1.0),vec4<f32>(0.3, 0.3, 0.8,1.0),noise_pixel);
        }
    }
    else if t == 56u // crushed_ice
    {
      col = mix(vec4<f32>(0.5, 0.8, 1.0, 2.0) * 0.5, vec4<f32>(0.5, 0.8, 1.5, 1.0), tdnoise);
      if (col.b > 1.0) {
        col = mix(vec4<f32>(0.8, 0.8, 1.1,1.0),vec4<f32>(0.3, 0.3, 0.8,1.0),noise_pixel);
      }
    }
    else if t == 57u // snow
    {
      col = mix(vec4<f32>(0.8, 0.9, 1.0, 2.0) * 0.8, vec4<f32>(0.8, 0.9, 2.0, 1.0), tdnoise);
      col.r = min(col.r, col.b);
      col.g = min(col.g, col.b);
      if (col.b > 1.0) {
        col = mix(vec4<f32>(0.8, 0.8, 1.1,1.0),vec4<f32>(0.3, 0.3, 0.8,1.0),noise_pixel);
      }
    }
    else if t == 60u // electricity
    {
      col = vec4<f32>(0.2, 0.4, 1.0, 1.0)*noise_pixel*noise_pixel*1.8;
    }
    else if t == 61u // plasma
    {
      col = vec4<f32>(0.6, 0.3, 1.0*noise_pixel*noise_pixel, 1.0)*noise_pixel*noise_pixel*1.8;
    }
    else if t == 62u // laser
    {
      col = vec4<f32>(1.0*tdnoise_faster + 1.5, 0.1, 0.0, 1.0)*noise_pixel*noise_pixel;
    }
    else if t == 6u // burning_wood
    {
      let flicker = clamp((tdnoise_fast + 1.0) * 0.5, 0.0, 1.0);
      col = mix(
        vec4<f32>(1.4, 0.35, 0.03, 1.0),  // darker ember orange
        vec4<f32>(2.8, 1.15, 0.12, 1.0),  // hotter bright orange/yellow
        flicker
      );
    }
    else if t == 7u
    {
      col = mix(
        vec4<f32>(8.0, 0.0, 0.0, 1.0),
        vec4<f32>(8.0, 0.5, 0.0, 1.0),
        noisy_mixer
      ) * 10.0;
    }
    else if t == 51u // burning powder
    {
      // Burning powder uses temperature for intense visual heat
      let n_fast = clamp(tdnoise_fast * 0.5 + 0.5, 0.0, 1.0);
      
      // Use temperature to drive the intensity - gunpowder gets EXTREMELY hot
      var heat01 = clamp((temp_value - 200.0) / (5000.0 - 200.0), 0.0, 1.0);
      heat01 = clamp(heat01 + (noise_pixel - 0.5) * 0.1, 0.0, 1.0);
      
      // Base color for burning powder (bright red-orange)
      let base_color = vec3<f32>(1.0, 0.3, 0.0);
      
      // Intense glow based on temperature
      let glow_rgb = fire_color_ramp(heat01);
      let flicker = clamp(0.90 + 0.30 * n_fast, 0.80, 1.40);
      // Much higher intensity multiplier for explosive heat
      let glow_intensity = mix(5.0, 25.0, heat01) * flicker;
      
      col = vec4<f32>(base_color + glow_rgb * glow_intensity, 1.0);
    }
    else if t == 8u // coal
    {
      col = vec4<f32>(0.05)*((noise_pixel+1.0)/3.0);
    }
    else if t == 9u
    {
      let a = clamp(0.16 + 0.12 * (noise_pixel * 0.5 + 0.5), 0.12, 0.30);
      col = vec4<f32>(0.0, 0.4, 0.0, a);
    }
    else if t == 10u // gas
    {
      let k = clamp((tdnoise_fast + 2.0) / 3.0, 0.0, 1.0);
      let a = clamp((0.08 + 0.22 * k) * (0.70 + 0.30 * (noise_pixel * 0.5 + 0.5)), 0.08, 0.35);
      col = vec4<f32>(vec3<f32>(0.2, 0.8, 0.2) * k, a);
    }
    else if t == 11u // burning gas
    {
      // Emissive overlay: add over the wall in the composite pass below.
      col = vec4<f32>(vec3<f32>(1.0, 0.8, 0.5) * 10.0 * tdnoise_fast, 1.0);
    }
    else if t == 83u // delute acid
    {
      let a = clamp(0.18 + 0.10 * (noise_pixel * 0.5 + 0.5), 0.12, 0.30);
      col = vec4<f32>(0.2, 0.6, 0.8, a);
    }
    else if t == 13u // salt
    {
      col = vec4<f32>(0.8,0.8,0.8,1.0);
    }
    else if t == 14u // base
    {
      col = vec4<f32>(1.0,0.2,0.2,1.0);
    }
    else if t == 84u // salty water
    {
      let a = clamp(0.12 + 0.10 * (noise_pixel * 0.5 + 0.5), 0.10, 0.25);
      col = vec4<f32>(0.5, 0.5, 1.0, a);
    }
    else if t == 85u // base water
    {
      let a = clamp(0.12 + 0.10 * (noise_pixel * 0.5 + 0.5), 0.10, 0.25);
      col = vec4<f32>(1.0, 0.5, 1.0, a);
    }
    else if t == 17u // liquid gas
    {
      // Cold bluish-green liquid.
      let lg = mix(vec4<f32>(0.3, 0.9, 0.7, 0.6) * 0.5, vec4<f32>(0.2, 0.8, 0.6, 0.8), tdnoise);
      let a = clamp(lg.a * 0.35, 0.10, 0.28);
      col = vec4<f32>(lg.rgb, a);
    }
    else if t == 70u // grass
    {
      col = vec4<f32>(0.4,1.0,0.4,1.0)*((noise_pixel+1.0)/2.0)*0.5;
    }
    else if t == 71u // dry grass
    {
      col = vec4<f32>(1.0,0.7,0.6,1.0)*((noise_pixel+1.0)/2.0)*0.15;
    }
    else if t == 80u // black hole
    {
      col = vec4<f32>(0.35, 0.0, 0.35, 1.0);
    }
    else if t == 20u // copper
    {
      // Copper color: RGB(184, 115, 51) normalized
      let base = vec3<f32>(0.72, 0.45, 0.20);
      let n = clamp((noise_pixel + 1.0) * 0.5, 0.0, 1.0);
      let rgb = base * (0.90 + 0.15 * n);
      col = vec4<f32>(rgb, 1.0);
    }
    else
    {
      col = vec4<f32>(0.0,1.0,0.0,1.0);
    }

    // Composite translucent fluids/gases over the wall (so bricks are visible through them).
    // Use alpha blending so dark colors are visible.
    if (is_translucent_fluid_or_gas(t)) {
        let a = clamp(col.a, 0.0, 1.0);
        col = vec4<f32>(mix(wall_bg_shadowed, col.rgb, a), 1.0);
    }

    // Direct heat visualization in Normal render (NOT bloom):
    // Blackbody-like heat tint driven by blurred temperature field.
    // Keep it subtle so it reads as "heat", not "paint".
    // Increased max temp to 5000 to allow explosive temperatures
    let heat = clamp((temp_vis - 140.0) / (5000.0 - 140.0), 0.0, 1.0);
    if (heat > 0.0 && settings.display_mode < 1.5) {
        // Exponential response (more perceptual): keeps low heat subtle, ramps high heat faster.
        let heat_e = 1.0 - exp(-2.6 * heat);

        let shimmer = clamp(tdnoise_fast * 0.5 + 0.5, 0.0, 1.0);

        // Feed the same ramp as fire, but clamp the top end so normal-mode heat
        // doesn't frequently go fully white.
        // Make it read like hot *material* (brick), not flame:
        // - a bit steeper response
        // - less animated noise
        var heat01 = clamp(pow(heat_e, 1.35) + (shimmer - 0.5) * 0.03, 0.0, 1.0);
        heat01 = min(heat01, 0.90);

        // Brick edge modulation: compute pseudo-normal from seam mask and make
        // brick edges heat up slightly stronger (only when the wall is visible).
        let wall_visible = (t == 0u) || is_translucent_fluid_or_gas(t);
        var edge_hot: f32 = 0.0;
        if (wall_visible) {
            let n = wall_brick_pseudo_normal(cell_xy);
            let seam = wall_brick_mortar_mask_simple(cell_xy);
            // Edge strength from the pseudo-normal tilt: 0 on flat areas, 1 on strong edges.
            let tilt = clamp((1.0 - n.z) * 7.0, 0.0, 1.0);
            edge_hot = clamp(max(seam, tilt) * (0.25 + 0.75 * heat_e), 0.0, 1.0);

            // Slightly push the color ramp and intensity at seams.
            heat01 = min(heat01 + 0.06 * edge_hot, 0.82);
        }

        let heat_rgb = brick_heat_color_ramp(heat01);

        // Strength: make falloff steeper (less "washed out" blobs).
        // We drive intensity with a stronger non-linearity than the color ramp:
        // - keeps the hot core smaller
        // - makes cooler outskirts fade exponentially faster
        let heat_falloff = pow(heat_e, 2.35);
        var heat_intensity = heat_falloff * mix(0.06, 0.14, shimmer);
        heat_intensity *= 1.0 + 0.35 * edge_hot;

        col = vec4<f32>(col.rgb + heat_rgb * heat_intensity, col.a);
    }

    // If in Both mode, blend temperature overlay with cell colors
    if settings.display_mode > 1.5 {
        // Blend temperature overlay: use additive blending with reduced intensity
        // Temperature overlay intensity: 0.5 (adjustable)
        let temp_intensity = 0.5;
        // Add only RGB; keep alpha from the base cell color.
        col = vec4<f32>(col.rgb + temp_col.rgb * temp_intensity, col.a);
        // Don't clamp here - allow bloom to work with values > 1.0
    }

    var out: FragmentOutput;
    
    // Calculate bloom before clamping (bloom needs values > 1.0)
    if (col.r > 1.0 || col.g > 1.0 || col.b > 1.0) {
        let maxc = max(col.r, max(col.g, col.b));
        let rgb = col.rgb / max(1.0, maxc);
        out.albedo = vec4<f32>(rgb, col.a);

        // Bloom: make response less "linear" (faster near highlights, less uniform).
        //
        // Soft knee + exponential curve:
        // - below threshold => 0
        // - knee region ramps smoothly
        // - above knee grows faster
        let threshold: f32 = 1.05;
        let knee: f32 = 0.65;
        let strength: f32 = 1.35;
        let gamma: f32 = 0.75; // <1 => more punch near threshold

        let over = maxc - threshold;
        let soft = clamp(over, 0.0, knee);
        let hard = max(over - knee, 0.0);
        let bloom_amt = (soft * soft) / max(1e-5, 2.0 * knee) + hard;

        // Map to [0..1) with a strong non-linear ramp.
        var k = 1.0 - exp(-strength * bloom_amt);
        k = pow(k, gamma);

        // Keep color energy proportional to highlight overshoot (helps "spread faster").
        let energy = clamp(bloom_amt, 0.0, 6.0);
        out.bloom = vec4<f32>(rgb * k * energy, 1.0);
    } else {
        out.albedo = col;
        out.bloom = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }

    return out;
}