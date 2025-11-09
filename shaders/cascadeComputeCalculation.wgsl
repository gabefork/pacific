@group(0) @binding(0) var<uniform> input: CascadeInput;
@group(1) @binding(0) var output: texture_storage_2d<rgba8unorm, write>;

const PI = 3.14159265;

struct CascadeInput {
    linear_sample_count: u32,
    angular_sample_count: u32,
    distance_between_probes: f32,
}

struct Object {
    pos: vec2<f32>,
    radius: f32,
    color: vec3<f32>,
    objectType: i32 // 0 for surface, 1 for light
}

fn sphereSDF(ray_position: vec2<f32>, light_pos: vec2<f32>, radius: f32) -> f32{
    return length(ray_position - light_pos) - radius;
}

const num_obj = 2;

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let angular_sample_count_sqrt = u32(sqrt(f32(input.angular_sample_count)));
    let world_pos = vec2f(id.xy) / 16.0;
    var sdfs: array<Object, num_obj>;

    // Define objects in scene here (don't forget to update num_obj)
    // Light 1
    sdfs[0].pos = vec2(0.5, 0.5);
    sdfs[0].radius = 0.2;
    sdfs[0].color = vec3(0.0);
    sdfs[0].objectType = 1;

    // Sphere
    sdfs[1].pos = vec2(0.5, 0.75);
    sdfs[1].radius = 0.3;
    sdfs[1].color = vec3(0.251, 0.529, 0.969);
    sdfs[1].objectType = 0;

    for (var i = 0u; i < input.angular_sample_count; i++) {
        let ray_radiance = cast_ray_in_direction(
            world_pos + vec2f(1.0/32.0),
            (f32(i) / f32(input.angular_sample_count) + 1.0/8.0) * (2.0 * PI),
            vec2f(0.0, 1.0/32.0),
            sdfs);
        let pix_pos = angular_sample_count_sqrt * vec2u(id.xy) + vec2u(i % angular_sample_count_sqrt, i / angular_sample_count_sqrt);
        textureStore(output, pix_pos, ray_radiance);
    }
}

fn cast_ray_in_direction(position: vec2f, angle: f32, interval: vec2f, objArr: array<Object, num_obj>) -> vec4f {

    // hard coded light at center
    // let light_pos = vec2f(0.5);
    // let light_radius = 0.2;
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

        // let distance = length(ray_pos - light_pos);
        // if (distance < light_radius) {
        //     hit_color = vec4f(1.0, 1.0, 0.8, 1.0);
        //     break;
        // }

        let closet_object = find_closest_object(ray_pos, objArr);
        let distance = sphereSDF(ray_pos, closet_object.pos, closet_object.radius);
        if (closet_object.objectType == 1 && distance < 0.0) {
            hit_color = vec4f(closet_object.color, 1.0);
            break;
        } else if (closet_object.objectType == 0 && distance < 0.0){
            hit_color = get_light_color(closet_object, objArr);
            break;
        }

        total_dist = total_dist + step_size;
    }

    return hit_color;
}

fn find_closest_object(ray_pos: vec2<f32>, objArr: array<Object, num_obj>) -> Object {
    var min_distance= sphereSDF(ray_pos, objArr[0].pos, objArr[0].radius);
    var min_obj: Object;
    for (var i = 1; i < num_obj; i++) {
        let distance = sphereSDF(ray_pos, objArr[i].pos, objArr[i].radius);
        if (distance < 0.0) {
            min_distance = distance;
            min_obj = objArr[i];
        }
    }
    return min_obj;
}

fn cast_ray(ray_origin: vec2<f32>, direction: vec2<f32>, number_of_steps: i32, step_size : f32, objArr: array<Object, num_obj>) {
    var total_dist = 0.;
    var hit_obj: Object;
    for (var i = 0; i < number_of_steps; i++) {
        var ray_pos = ray_origin + direction * total_dist;
        let distance = sphereSDF(ray_pos, objArr[i].pos, objArr[i].radius);

        if (distance < 0.0) {
            hit_obj = objArr[i];
            break;
        }

        total_dist = total_dist + step_size;
    }
    return hit_obj;
}

fn get_light_color(obj: Object, objArr: array<Object, num_obj>) -> vec4<f32>{
    return vec4(1.0);
}