use image::ImageBuffer;
use v4::{
    builtin_components::mesh_component::{MeshComponent, VertexDescriptor}, ecs::{
        compute::Compute,
        material::{ShaderAttachment, ShaderBufferAttachment, ShaderTextureAttachment},
    }, engine_support::texture_support::Texture, scene, V4
};
use wgpu::{StorageTextureAccess, TextureFormat};

use compute_texture_transfer_component::{ComputeTextureTransferComponent};

pub mod compute_texture_transfer_component;

#[tokio::main]
async fn main() {
    let mut engine = V4::builder()
        .window_settings(640, 640, "Pacific", None)
        .build()
        .await;

    let rendering_manager = engine.rendering_manager();
    let device = rendering_manager.device();

    let cascade_input = CascadeInput {
        linear_sample_count: 16,
        angular_sample_count: 16,
        distance_between_probes: 4.0,
    };
    let cascade_texture = Texture::create_texture(
        device,
        16 * 4,
        16 * 4,
        TextureFormat::Rgba8Unorm,
        Some(StorageTextureAccess::WriteOnly),
        false,
        wgpu::TextureUsages::COPY_SRC,
    );

    let mut buf = ImageBuffer::new(64, 64);

    for (_, _, pix) in buf.enumerate_pixels_mut() {
        *pix = image::Rgba([255; 4]);
    }

    scene! {
        scene: main_scene,
        active_camera: _,
        "display" = {
            material: {
                pipeline: {
                    fragment_shader_path: "shaders/radianceCascadeFrag.wgsl",
                    vertex_shader_path: "shaders/radianceCascadeVert.wgsl",
                    uses_camera: false,
                    vertex_layouts: [DisplayVert::vertex_layout()],
                },
                attachments: [
                    Texture(
                        texture: Texture::create_texture(device, 16 * 4, 16 * 4, TextureFormat::Rgba8Unorm, None, true, wgpu::TextureUsages::empty()),
                        visibility: wgpu::ShaderStages::FRAGMENT,
                    )
                ],
                // ident: "display",
            },
            components: [
                MeshComponent(
                    vertices: vec![vec![
                            DisplayVert {pos: [-1.0, 3.0, 0.0], tex_coords: [0.0, 2.0]},
                            DisplayVert {pos: [-1.0, -1.0, 0.0], tex_coords: [0.0, 0.0]},
                            DisplayVert {pos: [3.0, -1.0, 0.0], tex_coords: [2.0, 0.0]},
                        ]],
                    indices: vec![vec![0, 1, 2]],
                    enabled_models: vec![(0, None)],
                )
            ]
        },
        "probes" = {
            material: {
                pipeline: {fragment_shader_path: "shaders/tempFrag.wgsl", vertex_shader_path: "shaders/tempVert.wgsl", uses_camera: false},
                ident: "ignore"
            },
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
                    ident: "cascade_compute",
                )
            ],
            components: [
                ComputeTextureTransferComponent(compute_id: ident("cascade_compute"), ignore_material: ident("ignore"), texture_slot: 0)
            ]
        }
    };

    engine.attach_scene(main_scene);

    engine.main_loop().await;
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct DisplayVert {
    pos: [f32; 3],
    tex_coords: [f32; 2],
}

impl VertexDescriptor for DisplayVert {
    const ATTRIBUTES: &[wgpu::VertexAttribute] = &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];

    fn from_pos_normal_coords(pos: Vec<f32>, _normal: Vec<f32>, tex_coords: Vec<f32>) -> Self {
        Self {
            pos: pos.try_into().unwrap(),
            tex_coords: tex_coords.try_into().unwrap(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct CascadeInput {
    linear_sample_count: u32,
    angular_sample_count: u32,
    distance_between_probes: f32,
}
