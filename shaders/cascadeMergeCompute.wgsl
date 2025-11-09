@group(0) @binding(0) var<storage, read_write> input: MergeInput;
// number of outputs should be equal to id.z range ( = cascade levels)
@group(1) @binding(0) var output: texture_storage_2d<rgba8unorm, read_write>;
@group(2) @binding(0) var output2: texture_storage_2d<rgba8unorm, read_write>;
@group(3) @binding(0) var output3: texture_storage_2d<rgba8unorm, read_write>;

struct MergeInput {
    layer: u32,
    l0_probe_count: u32,
    l0_angular_sample_count: u32,
}

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) id: vec3u) {
    /* let resolution = input.l0_probe_count;
    // normalized world position in [0,1] range
    let world_pos = vec2f(id.xy) / f32(resolution);
    let lo_probe_count = resolution / u32(pow(2f, 2f * f32(input.layer)));
    // let probe_pos = (world_pos) * pow(2f, 2f * f32(cascade_level)) + vec2f(0.5/f32(resolution));
    let lo_probe_pos = (vec2f(id.xy) + vec2f(0.5)) / f32(lo_probe_count);

    // prune threads
    if (lo_probe_pos.x > 1.0 || lo_probe_pos.y > 1.0) { return; }

    
    let hi_probe_count = resolution / u32(pow(2f, 2f * f32(input.layer + 1u)));
    let hi_probe_pos_bl = (vec2f(id.xy) + vec2f(0.5)) / f32(hi_probe_count);
    let hi_probe_pos_br = (vec2f(id.xy) + vec2f(1.5, 0.5)) / f32(hi_probe_count); */

    let cur_angular_sample_count = input.l0_angular_sample_count / u32(pow(4f, f32(input.layer)));
    let hi_angular_sample_count = input.l0_angular_sample_count / u32(pow(4f, f32(input.layer + 1u)));
    for(var r = 0u; r < cur_angular_sample_count * cur_angular_sample_count; r++) {
        var hi_bl = id.xy / 2u;
        if(id.x % 4u <= 1u) {
            hi_bl -= vec2u(1u, 0u);
        }
        if(id.y % 4u <= 1u) {
            hi_bl -= vec2u(0u, 1u);
        }

        var hi_bl_sum = sum_nearby_rays(r, u32(sqrt(f32(hi_angular_sample_count))), hi_bl + vec2u(0u, 0u));
        var hi_br_sum = sum_nearby_rays(r, u32(sqrt(f32(hi_angular_sample_count))), hi_bl + vec2u(1u, 0u));
        var hi_tl_sum = sum_nearby_rays(r, u32(sqrt(f32(hi_angular_sample_count))), hi_bl + vec2u(0u, 1u));
        var hi_tr_sum = sum_nearby_rays(r, u32(sqrt(f32(hi_angular_sample_count))), hi_bl + vec2u(1u, 1u));
        let far = mix(mix(hi_bl_sum, hi_br_sum, fract(f32(id.x) / 4.0)), mix(hi_tl_sum, hi_tr_sum, fract(f32(id.x) / 4.0)), fract(f32(id.y) / 4.0));

        let near = current_ray_radiance(id.xy, r, u32(sqrt(f32(cur_angular_sample_count))));
        let radiance = near.xyz + (far.xyz * near.w);
    }

}

fn current_ray_radiance(probe_id: vec2u, ray_index: u32, lo_angular_sample_count_sqrt: u32) -> vec4f {
    switch input.layer {
        case 0u: {
            // TODO: Investigate
            return textureLoad(output, probe_id * 4u + offset_index_to_vector(0u, i32(ray_index), lo_angular_sample_count_sqrt));
        }
        case 1u: {
            return textureLoad(output2, probe_id * 4u + offset_index_to_vector(0u, i32(ray_index), lo_angular_sample_count_sqrt));
        }
        default: {
            return vec4f(0.0);
        }
    }
}

fn store_current_ray_radiance(probe_id: vec2u, ray_index: u32, lo_angular_sample_count_sqrt: u32, color: vec4f) {
    switch input.layer {
        case 0u: {
            // TODO: Investigate
            textureStore(output, probe_id * 4u + offset_index_to_vector(0u, i32(ray_index), lo_angular_sample_count_sqrt), color);
        }
        case 1u: {
            textureStore(output2, probe_id * 4u + offset_index_to_vector(0u, i32(ray_index), lo_angular_sample_count_sqrt), color);
        }
        default: {
            textureStore(output2, probe_id * 4u + offset_index_to_vector(0u, i32(ray_index), lo_angular_sample_count_sqrt), vec4f(0.0));
        }
    }
}

fn sum_nearby_rays(lo_ray_index: u32, hi_angular_sample_count_sqrt: u32, hi_probe_id: vec2u) -> vec4f {
    var ray_sum = vec4f(0.0);
    for(var i = -1; i <= 2; i++) {
        let hi_pix_pos = hi_probe_id * hi_angular_sample_count_sqrt + offset_index_to_vector(lo_ray_index, i, hi_angular_sample_count_sqrt);
        if(input.layer == 0) {
            ray_sum += textureLoad(output2, hi_pix_pos);

        } else {
            ray_sum += textureLoad(output3, hi_pix_pos);
        }
    }
    return ray_sum;
}

fn offset_index_to_vector(base_index: u32, offset: i32, hi_angular_sample_count_sqrt: u32) -> vec2u {
    let base = f32(4i * i32(base_index) + offset) % f32(hi_angular_sample_count_sqrt * hi_angular_sample_count_sqrt);
    return vec2u(u32(base % f32(hi_angular_sample_count_sqrt)), u32(base / f32(hi_angular_sample_count_sqrt)));
}
