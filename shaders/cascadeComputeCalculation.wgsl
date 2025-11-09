@group(0) @binding(0) var<uniform> input: CascadeInput;
// number of outputs should be equal to id.z range ( = cascade levels)
@group(1) @binding(0) var output: texture_storage_2d<rgba8unorm, write>;
@group(2) @binding(0) var output2: texture_storage_2d<rgba8unorm, write>;
@group(3) @binding(0) var output3: texture_storage_2d<rgba8unorm, write>;

const PI = 3.14159265;
// constant instead of passing because lior did not implement arrays in wgsl yet
// const CASCSADE_LEVELS = 3u;

struct CascadeInput {
    l0_probe_count: u32,
    angular_sample_count: u32,
    // distance_between_probes: f32,
    // cascade_levels: u32,
}

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let cascade_level = id.z;
    let resolution = input.l0_probe_count;

    // actual angular sample count for this cascade level
    let angular_sample_count = input.angular_sample_count << (2u * cascade_level);
    let angular_sample_count_sqrt = u32(sqrt(f32(angular_sample_count)));
    
    // normalized world position in [0,1] range
    let world_pos = vec2f(id.xy) / f32(resolution);
    let _probe_count = resolution / u32(pow(2f, 2f * f32(cascade_level)));
    // let probe_pos = (world_pos) * pow(2f, 2f * f32(cascade_level)) + vec2f(0.5/f32(resolution));
    let probe_pos = (vec2f(id.xy) + vec2f(0.5)) / f32(_probe_count);

    // prune threads
    if (probe_pos.x > 1.0 || probe_pos.y > 1.0) { return; }

    for (var i = 0u; i < angular_sample_count; i++) {
        let interval = 0.5/f32(resolution) * vec2f(interval_scale(cascade_level), interval_scale(cascade_level + 1u));
        let ray_radiance = cast_ray_in_direction(
            probe_pos,
            (f32(i) / f32(angular_sample_count) + 1.0/8.0) * (2.0 * PI),
            interval);
        
        let pix_pos = angular_sample_count_sqrt * vec2u(id.xy) + vec2u(i % angular_sample_count_sqrt, i / angular_sample_count_sqrt);
        
        // 0 scalability
        switch (cascade_level) {
            case 0u: {
                textureStore(output, pix_pos, ray_radiance);
            }
            case 1u: {
                textureStore(output2, pix_pos, ray_radiance);
            }
            case 2u: {
                textureStore(output3, pix_pos, ray_radiance);
            }
            default: {
                // do nothing
            }
        }
    }
}

fn interval_scale(cascade_level: u32) -> f32 {
    if (cascade_level == 0u) { return 0.0; }
    return f32(1u << (2u * cascade_level));
}

fn cast_ray_in_direction(position: vec2f, angle: f32, interval: vec2f) -> vec4f {

    // hard coded light at center
    let light_pos = vec2f(0.5);
    let light_radius = 0.2;
    // let light_radiosity = 10.0;
    
    // Ray marching
    let number_of_steps = 64u;
    let step_size = (interval.y-interval.x) / f32(number_of_steps);

    let dir = vec2f(cos(angle), sin(angle));
    let ray_origin = position + dir * interval.x;
    var total_dist = 0.;
    var hit_color = vec4f(0.0);
    for (var i = 0u; i < number_of_steps; i++) {
        var ray_pos = ray_origin + dir * total_dist;

        let distance = length(ray_pos - light_pos);
        if (distance < light_radius) {
            hit_color = vec4f(1.0, 1.0, 0.8, 1.0);
            break;
        }

        // wall collision
        if (ray_pos.x < 0.0) {
            hit_color = vec4f(0.0, 0.5, 0.5, 1.0);
            break;
        }
        if (ray_pos.x > 1.0) {
            hit_color = vec4f(1.0, 0.0, 0.0, 1.0);
            break;
        }
        if (ray_pos.y < 0.0) {
            hit_color = vec4f(0.0, 1.0, 0.0, 1.0);
            break;
        }
        if (ray_pos.y > 1.0) {
            hit_color = vec4f(1.0, 1.0, 0.0, 1.0);
            break;
        }

        total_dist = total_dist + step_size;
    }
    // let v = angle / (2.0 * PI);
    // return vec4f(position, v, 1.0);
    return hit_color;
}
