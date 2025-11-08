use v4::{
    V4,
    ecs::{
        compute::Compute,
        material::{ShaderAttachment, ShaderBufferAttachment, ShaderTextureAttachment},
    },
    engine_support::texture_support::Texture,
    scene,
};
use wgpu::{StorageTextureAccess, TextureFormat};

#[tokio::main]
async fn main() {
    let mut engine = V4::builder()
        .window_settings(800, 800, "Pacific", None)
        .build()
        .await;

    let rendering_manager = engine.rendering_manager();
    let device = rendering_manager.device();

    let cascade_input = CascadeInput {
        linear_sample_count: 16,
        angular_sample_count: 4,
        distance_between_probes: 4.0,
    };
    let cascade_texture = Texture::create_texture(
        device,
        16 * 2,
        16 * 2,
        TextureFormat::Rgba8Unorm,
        Some(StorageTextureAccess::WriteOnly),
        false,
    );

    scene! {
        scene: main_scene,
        active_camera: _,
        screen_space_materials: [
            {
                pipeline: {
                    fragment_shader_path: "shaders/radianceCascadeFrag.wgsl",
                },
                /* attachments: [
                    Texture(
                        texture: temp_texture,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                    )
                ] */
            },
        ],
        "probes" = {
            computes: [
                Compute(
                    input: vec![
                        ShaderAttachment::Buffer(ShaderBufferAttachment::new(
                            device, bytemuck::cast_slice(&[cascade_input]), wgpu::BufferBindingType::Uniform, wgpu::ShaderStages::COMPUTE, wgpu::BufferUsages::empty()
                        ))
                    ],
                    output: ShaderAttachment::Texture(ShaderTextureAttachment { texture: cascade_texture, visibility: wgpu::ShaderStages::COMPUTE, extra_usages: wgpu::TextureUsages::empty() }),
                    shader_path: "shaders/cascadeComputeCalculation.wgsl",
                    workgroup_counts: (16, 16, 1),
                )
            ]
        }
    };

    engine.attach_scene(main_scene);

    engine.main_loop().await;
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct CascadeInput {
    linear_sample_count: u32,
    angular_sample_count: u32,
    distance_between_probes: f32,
}
