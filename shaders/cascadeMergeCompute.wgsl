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

struct Thing {
    @builtin(global_invocation_id) id: vec3u,
    @builtin(local_invocation_id) r: vec3u,
}

@compute
@workgroup_size(64)
fn main(in: Thing) {
    let cur_angular_sample_count = input.l0_angular_sample_count / u32(pow(4f, f32(input.layer)));
    let cur_angular_sample_count_sqrt = u32(sqrt(f32(cur_angular_sample_count)));
    let hi_angular_sample_count = input.l0_angular_sample_count / u32(pow(4f, f32(input.layer + 1u)));
    let hi_angular_sample_count_sqrt = u32(sqrt(f32(hi_angular_sample_count)));

    let id = in.id;
    let r = in.r.y * 4u + in.r.x;

    // for(var r = 0u; r < cur_angular_sample_count * cur_angular_sample_count; r++) {
        var hi_bl = id.xy / 2u;
        if(id.x % 4u <= 1u) {
            hi_bl -= vec2u(1u, 0u);
        }
        if(id.y % 4u <= 1u) {
            hi_bl -= vec2u(0u, 1u);
        }

        var hi_bl_sum = sum_nearby_rays(r, hi_angular_sample_count_sqrt, hi_bl + vec2u(0u, 0u));
        // let far = hi_bl_sum;

        /* let near = current_ray_radiance(id.xy, r, cur_angular_sample_count_sqrt);
        let radiance = near.rgb + (far.rgb * near.a);
        store_current_ray_radiance(id.xy, r, cur_angular_sample_count_sqrt, vec4f(radiance, near.a * far.a)); */
        var hi_br_sum = sum_nearby_rays(r, hi_angular_sample_count_sqrt, hi_bl + vec2u(1u, 0u));
        var hi_tl_sum = sum_nearby_rays(r, hi_angular_sample_count_sqrt, hi_bl + vec2u(0u, 1u));
        var hi_tr_sum = sum_nearby_rays(r, hi_angular_sample_count_sqrt, hi_bl + vec2u(1u, 1u));
        let far = mix(mix(hi_bl_sum, hi_br_sum, fract(f32(id.x) / 4.0)), mix(hi_tl_sum, hi_tr_sum, fract(f32(id.x) / 4.0)), fract(f32(id.y) / 4.0));

        let near = current_ray_radiance(id.xy, r, cur_angular_sample_count_sqrt);
        let radiance = near.rgb + (far.rgb * near.a);
        store_current_ray_radiance(id.xy, r, cur_angular_sample_count_sqrt, vec4f(radiance, near.a * far.a));
    // }

    if(all(id.xy == vec2(0u)) && r == 0u) {
        if (input.layer > 0u) {
            input.layer -= 1u;
        } else {
            input.layer = 1u;
        }

    }
}

fn current_ray_radiance(probe_id: vec2u, ray_index: u32, lo_angular_sample_count_sqrt: u32) -> vec4f {
    switch input.layer {
        case 0u: {
            return textureLoad(output, probe_id * lo_angular_sample_count_sqrt + offset_to_vector(i32(ray_index), lo_angular_sample_count_sqrt));
        }
        case 1u: {
            return textureLoad(output2, probe_id * lo_angular_sample_count_sqrt + offset_to_vector(i32(ray_index), lo_angular_sample_count_sqrt));
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
            textureStore(output, probe_id * lo_angular_sample_count_sqrt + offset_to_vector(i32(ray_index), lo_angular_sample_count_sqrt), color);
        }
        case 1u: {
            textureStore(output2, probe_id *lo_angular_sample_count_sqrt + offset_to_vector(i32(ray_index), lo_angular_sample_count_sqrt), color);
        }
        default: {
            textureStore(output2, probe_id *lo_angular_sample_count_sqrt  + offset_to_vector(i32(ray_index), lo_angular_sample_count_sqrt), vec4f(0.0));
        }
    }
}

fn sum_nearby_rays(lo_ray_index: u32, hi_angular_sample_count_sqrt: u32, hi_probe_id: vec2u) -> vec4f {
    var ray_sum = vec4f(0.0);
    for(var i = -1; i <= 2; i++) {
        let hi_pix_pos = hi_probe_id * hi_angular_sample_count_sqrt + offset_index_to_vector(lo_ray_index, i, hi_angular_sample_count_sqrt);
        if(input.layer == 0u) {
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


fn offset_to_vector(offset: i32, hi_angular_sample_count_sqrt: u32) -> vec2u {
    return vec2u(u32(f32(offset) % f32(hi_angular_sample_count_sqrt)), u32(f32(offset) / f32(hi_angular_sample_count_sqrt)));
}
