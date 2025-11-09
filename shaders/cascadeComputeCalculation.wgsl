@group(0) @binding(0) var<uniform> input: CascadeInput;
@group(1) @binding(0) var<uniform> obj_arr: array<Object, NUM_OBJ>;
@group(2) @binding(0) var output: texture_storage_2d<rgba8unorm, write>;
@group(3) @binding(0) var output2: texture_storage_2d<rgba8unorm, write>;

const PI = 3.14159265;
const NUM_OBJ = 2;
// constant instead of passing because lior did not implement arrays in wgsl yet
const CASCSADE_LEVELS = 2u;

struct CascadeInput {
    l0_probe_count: u32,
    angular_sample_count: u32,
    // distance_between_probes: f32,
    // cascade_levels: u32,
}

struct Object {
    pos: vec2<f32>,
    radius: f32,
    objectType: i32, // 0 for surface, 1 for light
    color: vec3<f32>,
    data: f32,
}

struct RaycastResult {
    object_index: i32,
    distance: f32,
    intersection_pos: vec2<f32>,
}

fn sphereSDF(ray_position: vec2<f32>, pos: vec2<f32>, radius: f32) -> f32{
    return distance(ray_position, pos) - radius;
}

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    // order in lowest cascade to highest cascade
    // var output_level = array<texture_storage_2d<rgba8unorm, write>, CASCSADE_LEVELS>(output, output2);
    let cascade_level = id.z;
    let resolution = input.l0_probe_count;

    // actual angular sample count for this cascade level
    let angular_sample_count = input.angular_sample_count << (2u * cascade_level);
    let angular_sample_count_sqrt = u32(sqrt(f32(angular_sample_count)));
    
    // normalized world position in [0,1] range
    let world_pos = vec2f(id.xy) / f32(resolution);
    let _probe_count = resolution / u32(pow(2f, 2f * f32(cascade_level)));
    let probe_pos = (world_pos) * pow(2f, 2f * f32(cascade_level)) + vec2f(0.5/f32(resolution));

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

    // Ray marching
    let ray_direction = vec2f(cos(angle), sin(angle));
    let ray_origin = position + ray_direction * interval.x;
    let max_distance = interval.y - interval.x;

    let ray_result = cast_ray(ray_origin, ray_direction, max_distance, -1);
    if(ray_result.object_index == -1) {
        return vec4f(0.0);
    }

    let ray_intersection_object = obj_arr[ray_result.object_index];
    switch ray_intersection_object.objectType {
        case 0: {
            return vec4f(ray_intersection_object.color, 1.0) /* * get_light_attenuation(ray_intersection_object.pos, max_distance, ray_result.object_index) */;
        }
        case 1: {
            return ray_intersection_object.data * vec4f(ray_intersection_object.color, 1.0);
        }
        default {
            return vec4f(0.0);
        }
    }
}

fn find_closest_object(ray_pos: vec2<f32>, excluded_index: i32) -> u32 {
    var min_distance= sphereSDF(ray_pos, obj_arr[0].pos, obj_arr[0].radius);
    var min_obj = 0u;
    for (var i = 1u; i < u32(NUM_OBJ); i++) {
        if(excluded_index != -1 && i == u32(excluded_index)) {
            continue;
        }
        let distance = sphereSDF(ray_pos, obj_arr[i].pos, obj_arr[i].radius);
        if (distance < min_distance) {
            min_distance = distance;
            min_obj =  i;
        }
    }
    return min_obj;
}

fn cast_ray(ray_origin: vec2<f32>, direction: vec2<f32>, max_distance: f32, excluded_index: i32) -> RaycastResult {
    var total_dist = 0.;

    var hit_object: RaycastResult;
    hit_object.object_index = -1;
    while (total_dist < max_distance) {
        var ray_pos = ray_origin + direction * total_dist;

        let closest_object_index = find_closest_object(ray_pos, excluded_index);
        let closest_object = obj_arr[closest_object_index];
        let distance = sphereSDF(ray_pos, closest_object.pos, closest_object.radius);
        if (distance <= 0.0) {
            hit_object.object_index = i32(closest_object_index);
            hit_object.distance = total_dist;
            break;
        }
        total_dist = total_dist + distance;
    }
    return hit_object;
}

fn get_light_attenuation(pos: vec2f, max_distance: f32, origin_obj_index: i32) -> vec4<f32>{
    var accumulated_light = vec4f(0.0);

    for(var i = 0; i < NUM_OBJ; i++) {
        let obj = obj_arr[i];
        if(obj.objectType != 1) {
            continue;
        }

        let raycast_result = cast_ray(pos, obj_arr[i].pos - pos, max_distance, origin_obj_index);
        if(raycast_result.object_index == -1) {
            continue;
        }

        let raycast_obj = obj_arr[raycast_result.object_index];
        accumulated_light += vec4f(f32(raycast_result.object_index), f32(raycast_result.object_index), 0.0, 1.0);
        if(raycast_obj.objectType == 1) {
            accumulated_light += vec4f(1.0, 0.0, 0.0, 1.0) /* raycast_obj.data / raycast_result.distance */ /* * vec4f(obj_arr[raycast_result.object_index].color, 1.0) */;
        }
    }

    return accumulated_light;
}
