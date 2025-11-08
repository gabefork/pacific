@group(0) @binding(0) var<uniform> input: CascadeInput;
@group(1) @binding(0) var output: texture_storage_2d<rgba8unorm, write>;

const PI = 3.14159265;

struct CascadeInput {
    linear_sample_count: u32,
    angular_sample_count: u32,
    distance_between_probes: f32,
}

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let angular_sample_count_sqrt = u32(sqrt(f32(input.angular_sample_count)));
    for (var i = 0u; i < input.angular_sample_count; i++) {
        let ray_radiance = cast_ray_in_direction(vec2f(id.xy) * input.distance_between_probes, f32(i) / f32(input.angular_sample_count) * (2.0 * PI) / f32(input.angular_sample_count), vec2f(0.0));
        let pix_pos = vec2u(angular_sample_count_sqrt,angular_sample_count_sqrt) * vec2u(id.xy) + vec2u(i % angular_sample_count_sqrt, i / angular_sample_count_sqrt);
        textureStore(output, pix_pos, ray_radiance);
    }
}

fn cast_ray_in_direction(position: vec2f, angle: f32, offset: vec2f) -> vec4f {
    let v = angle % (2.0 * PI);
    return vec4f(v, v, v, 1.0);
}
