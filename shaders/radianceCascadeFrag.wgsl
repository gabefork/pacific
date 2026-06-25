@group(0) @binding(0) var cascade: texture_2d<f32>;
@group(0) @binding(1) var sample: sampler;

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>
}

struct Cascade {
    angular_distance: f32,
    angular_resolution: i32,
    x_pos: f32,
    y_pos: f32
}

struct Ray {
    direction: vec3<f32>,
    magnitude: vec2<f32>,
    x_pos: f32,
    y_pos: f32
}

const LINEAR_PROBE_COUNT = 128u;
const RADIAL_SAMPLE_COUNT_SQRT = 16u;

@fragment
fn main(in: VertexOut) -> @location(0) vec4f { 
    return get_fluence(in.tex_coords);
}

fn get_radiance_from_probe(probe_bottom_left_coord: vec2f) -> vec4f {
    var sum = vec4f(0.0);
    for (var i = 0; i < 4; i++) {
        for (var j = 0; j < 4; j++) {
            sum += textureSample(cascade, sample, probe_bottom_left_coord + vec2f(f32(i), f32(j)) / f32(LINEAR_PROBE_COUNT * RADIAL_SAMPLE_COUNT_SQRT));
        }
    }

    return sum;
}

fn get_fluence(pos: vec2f) -> vec4f {
    let bottom_left_coord = floor(pos * f32(LINEAR_PROBE_COUNT)) / f32(LINEAR_PROBE_COUNT);

    var fluence = vec4f(0.0);
    for (var i = 0; i < 2; i++) {
        for (var j = 0; j < 2; j++) {
            fluence += get_radiance_from_probe(bottom_left_coord + vec2f(f32(i), f32(j)) / 32.0);
        }
    }

    return fluence;
}
