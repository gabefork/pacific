struct VertexOut {
    @builtin(position) position: vec4<f32>,
}

@fragment
fn main(in: VertexOut) -> @location(0) vec4<f32> {
    let color = vec3f(0.0, 1.0, 0.0);
    return vec4f(color, 1.0);
}
