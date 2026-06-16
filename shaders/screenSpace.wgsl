struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) frag_uv: vec2<f32>,
};

@group(0) @binding(0) var input_tex: texture_2d<f32>;
@group(1) @binding(0) var input_sampler: sampler;

@fragment
fn main(input: VertexOutput) -> @location(0) vec4<f32> {
    let uv = input.frag_uv;
    let color = textureLoad(input_tex, vec2i(uv * vec2f(textureDimensions(input_tex))), 0);
    return color;
}