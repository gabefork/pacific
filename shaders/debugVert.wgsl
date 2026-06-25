struct VertexIn {
    @location(0) position: vec3<f32>,
}

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
}

@vertex
fn main(model: VertexIn) -> VertexOut {
    var out: VertexOut;
    out.clip_position = vec4f(model.position, 1.0);
    return out;
}
