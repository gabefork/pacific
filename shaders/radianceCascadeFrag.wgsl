struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>
}

struct Cascade {
    angular_distance: f32,
    angular_resolution: i32
    x_pos: f32,
    y_pos: f32
}

struct Ray {
    direction: vec3<f32>,
    magnitude: vec2<f32>,
    x_pos: f32,
    y_pos: f32
}

@fragment
fn main(vertex_output: VertexOut) -> @location(0) vec4f { 
    return out.clip_position;
}