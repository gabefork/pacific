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
        .limits(wgpu::Limits{max_bind_groups: 8, ..Default::default()})
        .features(wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES)
        .build()
        .await;

    let rendering_manager = engine.rendering_manager();
    let device = rendering_manager.device();

    // actually square root of probe count
    const HIGHEST_PROBE_CNT: u32 = 128;
    const ANGULAR_COUNT_SQRT: u32 = 8;
    let cascade_input = CascadeInput {
        l0_probe_count: HIGHEST_PROBE_CNT,
        angular_sample_count: ANGULAR_COUNT_SQRT * ANGULAR_COUNT_SQRT,
        // distance_between_probes: 4.0,
        // cascade_levels: 2,
    };
    let cascade_texture = Texture::create_texture(
        device,
        HIGHEST_PROBE_CNT * ANGULAR_COUNT_SQRT,
        HIGHEST_PROBE_CNT * ANGULAR_COUNT_SQRT,
        TextureFormat::Rgba8Unorm,
        Some(StorageTextureAccess::WriteOnly),
        false,
        wgpu::TextureUsages::COPY_SRC,
    );
    let cascade_texture2 = Texture::create_texture(
        device,
        HIGHEST_PROBE_CNT * ANGULAR_COUNT_SQRT / 2,
        HIGHEST_PROBE_CNT * ANGULAR_COUNT_SQRT / 2,
        TextureFormat::Rgba8Unorm,
        Some(StorageTextureAccess::WriteOnly),
        false,
        wgpu::TextureUsages::COPY_SRC,
    );
    let cascade_texture3 = Texture::create_texture(
        device,
        HIGHEST_PROBE_CNT * ANGULAR_COUNT_SQRT / 4,
        HIGHEST_PROBE_CNT * ANGULAR_COUNT_SQRT / 4,
        TextureFormat::Rgba8Unorm,
        Some(StorageTextureAccess::WriteOnly),
        false,
        wgpu::TextureUsages::COPY_SRC,
    );

    let cascade_texture_merge = Texture::create_texture(
        device,
        HIGHEST_PROBE_CNT * ANGULAR_COUNT_SQRT,
        HIGHEST_PROBE_CNT * ANGULAR_COUNT_SQRT,
        TextureFormat::Rgba8Unorm,
        Some(StorageTextureAccess::ReadWrite),
        false,
        wgpu::TextureUsages::COPY_SRC,
    );
    let cascade_texture2_merge = Texture::create_texture(
        device,
        HIGHEST_PROBE_CNT * ANGULAR_COUNT_SQRT / 2,
        HIGHEST_PROBE_CNT * ANGULAR_COUNT_SQRT / 2,
        TextureFormat::Rgba8Unorm,
        Some(StorageTextureAccess::ReadWrite),
        false,
        wgpu::TextureUsages::COPY_SRC,
    );
    let cascade_texture3_merge = Texture::create_texture(
        device,
        HIGHEST_PROBE_CNT * ANGULAR_COUNT_SQRT / 4,
        HIGHEST_PROBE_CNT * ANGULAR_COUNT_SQRT / 4,
        TextureFormat::Rgba8Unorm,
        Some(StorageTextureAccess::ReadWrite),
        false,
        wgpu::TextureUsages::COPY_SRC,
    );

    let merging_input = MergingInput{
        current_level: 2,
        l0_probe_count: HIGHEST_PROBE_CNT,
        l0_angular_sample_count: ANGULAR_COUNT_SQRT * ANGULAR_COUNT_SQRT,
    };

    let surface_objects = vec![
        SurfaceObject {pos: [0.2, 0.2], radius: 0.2, color: [1.0; 3], object_type: 1, data: 0.2},
        SurfaceObject {pos: [0.5, 0.75], radius: 0.3, color: [0.251, 0.529, 0.969], object_type: 0, data: 0.0},
        SurfaceObject {pos: [1.0, 0.0], radius: 0.05, color: [0.8, 0.0, 0.0], object_type: 1, data: 0.2},
    ];

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
                        texture: Texture::create_texture(device, HIGHEST_PROBE_CNT * ANGULAR_COUNT_SQRT, HIGHEST_PROBE_CNT * ANGULAR_COUNT_SQRT, TextureFormat::Rgba8Unorm, None, true, wgpu::TextureUsages::empty()),
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
                        )),
                        ShaderAttachment::Buffer(ShaderBufferAttachment::new(
                            device, bytemuck::cast_slice(&surface_objects), wgpu::BufferBindingType::Uniform, wgpu::ShaderStages::COMPUTE, wgpu::BufferUsages::empty()
                        )),
                        // fix this later
                        // order of 'output's follow order of textures passed in
                        ShaderAttachment::Texture(ShaderTextureAttachment { texture: cascade_texture.clone(), visibility: wgpu::ShaderStages::COMPUTE, extra_usages: wgpu::TextureUsages::empty() }),
                        ShaderAttachment::Texture(ShaderTextureAttachment { texture: cascade_texture2.clone(), visibility: wgpu::ShaderStages::COMPUTE, extra_usages: wgpu::TextureUsages::empty() }),
                        ShaderAttachment::Texture(ShaderTextureAttachment { texture: cascade_texture3.clone(), visibility: wgpu::ShaderStages::COMPUTE, extra_usages: wgpu::TextureUsages::empty() }),
                    ],
                    // output: ShaderAttachment::Texture(ShaderTextureAttachment { texture: cascade_texture3.clone(), visibility: wgpu::ShaderStages::COMPUTE, extra_usages: wgpu::TextureUsages::empty() }),
                    shader_path: "shaders/cascadeComputeCalculation.wgsl",
                    workgroup_counts: (HIGHEST_PROBE_CNT, HIGHEST_PROBE_CNT, 3),
                    ident: "cascade_compute",
                ),
                Compute(
                    input: vec![
                        ShaderAttachment::Buffer(ShaderBufferAttachment::new(
                            device, bytemuck::cast_slice(&[merging_input]), wgpu::BufferBindingType::Storage { read_only: false }, wgpu::ShaderStages::COMPUTE, wgpu::BufferUsages::empty()
                        )),
                        ShaderAttachment::Texture(ShaderTextureAttachment { texture: cascade_texture_merge, visibility: wgpu::ShaderStages::COMPUTE, extra_usages: wgpu::TextureUsages::empty() }),
                        ShaderAttachment::Texture(ShaderTextureAttachment { texture: cascade_texture2_merge, visibility: wgpu::ShaderStages::COMPUTE, extra_usages: wgpu::TextureUsages::empty() }),
                        ShaderAttachment::Texture(ShaderTextureAttachment { texture: cascade_texture3_merge, visibility: wgpu::ShaderStages::COMPUTE, extra_usages: wgpu::TextureUsages::empty() }),
                    ],
                    // output: ShaderAttachment::Texture(ShaderTextureAttachment { texture: cascade_texture3_merge, visibility: wgpu::ShaderStages::COMPUTE, extra_usages: wgpu::TextureUsages::empty() }),
                    shader_path: "shaders/cascadeMergeCompute.wgsl",
                    iterate_count: 2,
                    workgroup_counts: (HIGHEST_PROBE_CNT, HIGHEST_PROBE_CNT, 1),
                    ident: "cascade_merge",
                )
            ],
            components: [
                ComputeTextureTransferComponent(compute_id: ident("cascade_compute"), merge_id: ident("cascade_merge"), ignore_material: ident("ignore"), texture_slot: 0)
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
    l0_probe_count: u32,
    angular_sample_count: u32,
    // distance_between_probes: f32,
    // cascade_levels: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct SurfaceObject {
    pos: [f32; 2],
    radius: f32,
    object_type: i32, // 0 for surface, 1 for light
    color: [f32; 3],
    data: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct MergingInput {
    current_level: u32,
    l0_probe_count: u32,
    l0_angular_sample_count: u32,
}
