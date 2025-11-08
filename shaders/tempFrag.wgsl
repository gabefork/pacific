@group(0) @binding(0) var cascade: texture_2d<f32>;
@group(0) @binding(1) var sample: sampler;


@fragment
fn main(@builtin(position) clip_position: vec4<f32>) -> @location(0) vec4f { 
    return vec4f(0.0);
}
