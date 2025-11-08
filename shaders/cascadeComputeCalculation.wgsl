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
    let world_pos = vec2f(id.xy) / 16.0;
    for (var i = 0u; i < input.angular_sample_count; i++) {
        let ray_radiance = cast_ray_in_direction(
            world_pos + vec2f(1.0/32.0),
            (f32(i) / f32(input.angular_sample_count) + 0.5) * (2.0 * PI),
            vec2f(0.0, 1.0/32.0));
        let pix_pos = angular_sample_count_sqrt * vec2u(id.xy) + vec2u(i % angular_sample_count_sqrt, i / angular_sample_count_sqrt);
        textureStore(output, pix_pos, ray_radiance);
    }
}

fn cast_ray_in_direction(position: vec2f, angle: f32, interval: vec2f) -> vec4f {

    // hard coded light at center
    let light_pos = vec2f(0.5);
    let light_radius = 0.1;
    // let light_radiosity = 10.0;
    
    // Ray marching
    let number_of_steps = 32u;
    let step_size = (interval.y-interval.x) / f32(number_of_steps);

    let ray_direction = vec2f(cos(angle), sin(angle));
    let ray_origin = position + ray_direction * interval.x;
    var total_dist = 0.;
    var hit_color = vec4f(0.0);
    for (var i = 0u; i < number_of_steps; i++) {
        var ray_pos = ray_origin + ray_direction * total_dist;

        let distance = length(ray_pos - light_pos);
        if (distance < light_radius) {
            hit_color = vec4f(1.0, 1.0, 0.8, 1.0);
            break;
        }
    }

    return hit_color;
}
