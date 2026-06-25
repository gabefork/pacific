use v4::{
    V4, builtin_components::mesh_component::{MeshComponent, VertexData, VertexDescriptor}, ecs::{
        compute::Compute, material::{ShaderAttachment, ShaderBufferAttachment, ShaderTextureAttachment},
    }, engine_support::texture_support::{TextureBundle, TextureProperties}, scene
};
use wgpu::{StorageTextureAccess, TextureFormat};
use winit::{
    dpi::PhysicalSize,
    window::WindowAttributes,
};
use nalgebra::vector;

use compute_texture_transfer_component::{ComputeTextureTransferComponent};

pub mod compute_texture_transfer_component;

// debug macro, toggles debug code
const DEBUG: bool = false;
macro_rules! debug_only {
    ($($code:tt)*) => {
        if DEBUG
        { $($code)*}
    };
}

#[tokio::main]
async fn main() {
    let mut engine = V4::builder()
        .window_attributes(
            WindowAttributes::default()
                .with_surface_size(PhysicalSize::new(640, 640))
                .with_title("Pacific")
        )
        .limits(wgpu::Limits{max_bind_groups: 8, ..Default::default()})
        .features(wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES | wgpu::Features::POLYGON_MODE_LINE)
        .build()
        .await;

    let rendering_manager = engine.rendering_manager();
    let device = rendering_manager.device();

    // remember to match these values in shaders/radianceCascadeFrag.wgsl
    // actually square root of probe count
    const HIGHEST_PROBE_CNT: u32 = 128;
    const ANGULAR_COUNT_SQRT: u32 = 16;
    const DIM: u32 = HIGHEST_PROBE_CNT * ANGULAR_COUNT_SQRT;
    let cascade_input = CascadeInput {
        l0_probe_count: HIGHEST_PROBE_CNT,
        angular_sample_count: ANGULAR_COUNT_SQRT * ANGULAR_COUNT_SQRT,
        // distance_between_probes: 4.0,
        // cascade_levels: 2,
    };
    let (_, cascade_texture_bundle) = TextureBundle::create_texture(
        device,
        DIM,
        DIM,
        TextureProperties {
            format: TextureFormat::Rgba8Unorm,
            storage_texture: Some(StorageTextureAccess::WriteOnly),
            is_sampled: false,
            extra_usages: wgpu::TextureUsages::COPY_SRC,
            ..Default::default()
        }
    );
    let (_, cascade_texture2_bundle) = TextureBundle::create_texture(
        device,
        DIM / 2,
        DIM / 2,
        TextureProperties {
            format: TextureFormat::Rgba8Unorm,
            storage_texture: Some(StorageTextureAccess::WriteOnly),
            is_sampled: false,
            extra_usages: wgpu::TextureUsages::COPY_SRC,
            ..Default::default()
        }
    );
    let (_, cascade_texture3_bundle) = TextureBundle::create_texture(
        device,
        DIM / 4,
        DIM / 4,
        TextureProperties {
            format: TextureFormat::Rgba8Unorm,
            storage_texture: Some(StorageTextureAccess::WriteOnly),
            is_sampled: false,
            extra_usages: wgpu::TextureUsages::COPY_SRC,
            ..Default::default()
        }
    );

    let (_, cascade_texture_merge_bundle) = TextureBundle::create_texture(
        device,
        DIM,
        DIM,
        TextureProperties {
            format: TextureFormat::Rgba8Unorm,
            storage_texture: Some(StorageTextureAccess::ReadWrite),
            is_sampled: false,
            extra_usages: wgpu::TextureUsages::COPY_SRC,
            ..Default::default()
        }
    );
    let (_, cascade_texture2_merge_bundle) = TextureBundle::create_texture(
        device,
        DIM / 2,
        DIM / 2,
        TextureProperties {
            format: TextureFormat::Rgba8Unorm,
            storage_texture: Some(StorageTextureAccess::ReadWrite),
            is_sampled: false,
            extra_usages: wgpu::TextureUsages::COPY_SRC,
            ..Default::default()
        }
    );
    let (_, cascade_texture3_merge_bundle) = TextureBundle::create_texture(
        device,
        DIM / 4,
        DIM / 4,
        TextureProperties {
            format: TextureFormat::Rgba8Unorm,
            storage_texture: Some(StorageTextureAccess::ReadWrite),
            is_sampled: false,
            extra_usages: wgpu::TextureUsages::COPY_SRC,
            ..Default::default()
        }
    );

    let merging_input = MergingInput{
        current_level: 1,
        l0_probe_count: HIGHEST_PROBE_CNT,
        l0_angular_sample_count: ANGULAR_COUNT_SQRT * ANGULAR_COUNT_SQRT,
    };

    // objects in the scene
    let surface_objects = vec![
        SurfaceObject {pos: [0.2, 0.2], radius: 0.2, color: [1.0; 3], object_type: 1, data: 0.2}, // white
        SurfaceObject {pos: [0.5, 0.75], radius: 0.3, color: [0.251, 0.529, 0.969], object_type: 0, data: 0.05}, // blue ??
        SurfaceObject {pos: [1.0, 0.0], radius: 0.05, color: [0.8, 0.0, 0.0], object_type: 1, data: 0.08}, // red
    ];

    let (_, display_bundle) = TextureBundle::create_texture(
        device,
        DIM,
        DIM,
        TextureProperties {
            format: TextureFormat::Rgba8Unorm,
            storage_texture: None,
            is_sampled: true,
            extra_usages: wgpu::TextureUsages::empty(),
            ..Default::default()
        }
    );

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
                    Texture (
                        texture_bundle: display_bundle,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                    ),
                ],
                // ident: "display",
            },
            components: [
                MeshComponent(
                    vertices: vec![vec![
                            DisplayVert {pos: [-1.0, 3.0, 1.0], tex_coords: [0.0, 2.0]},
                            DisplayVert {pos: [-1.0, -1.0, 1.0], tex_coords: [0.0, 0.0]},
                            DisplayVert {pos: [3.0, -1.0, 1.0], tex_coords: [2.0, 0.0]},
                        ]],
                    indices: vec![vec![0, 1, 2]],
                    enabled_models: vec![(0, None)],
                )
            ]
        },
        "debug" = { // debug lines for probe rays
            material: {
                pipeline: {
                    fragment_shader_path: "shaders/debugFrag.wgsl",
                    vertex_shader_path: "shaders/debugVert.wgsl",
                    uses_camera: false,
                    vertex_layouts: [DebugVert::vertex_layout()],
                    geometry_details: {
                        topology: wgpu::PrimitiveTopology::LineList,
                        cull_mode: None,
                        polygon_mode: wgpu::PolygonMode::Fill,
                    }
                },
            },
            components: [
                MeshComponent(
                    vertices: if DEBUG { vec![
                        (0..HIGHEST_PROBE_CNT/4).flat_map(|x| {
                        (0..HIGHEST_PROBE_CNT/4).flat_map(move |y| {
                        (0..3).flat_map(move |z| {
                            // actual angular sample count for this cascade level
                            let angular_sample_count = cascade_input.angular_sample_count << (2*z);
                            println!("AAAAA");
                            
                            let _probe_count = HIGHEST_PROBE_CNT/4 / u32::pow(2, 2 * z);
                            // normalize probe position to [-1, 1] range
                            let probe_pos = vector![x as f32, y as f32] * 2.0 / _probe_count as f32 - vector![1.0, 1.0];
                            fn interval_scale(cascade_level: u32) -> f32 {
                                if cascade_level == 0 { return 0.0; }
                                return (1 << (2 * cascade_level)) as f32;
                            }

                            (0..angular_sample_count).flat_map(move |i| {
                                let interval = 0.5/4.0/HIGHEST_PROBE_CNT as f32 * vector![interval_scale(z), interval_scale(z+1)];
                                let angle = (i as f32 / angular_sample_count as f32) * (2.0 * std::f32::consts::PI);

                                let (s,c) = angle.sin_cos();
                                let ray_direction = vector![c, s];
                                let ray_origin = probe_pos + ray_direction * interval.x;
                                let ray_end = probe_pos + ray_direction * interval.y;
                                vec![
                                    DebugVert { pos: [ray_origin.x, ray_origin.y, z as f32/3.0] },
                                    DebugVert { pos: [ray_end.x, ray_end.y, z as f32/3.0] }
                                ]
                            }).collect::<Vec<_>>()
                        }).collect::<Vec<_>>()}).collect::<Vec<_>>()}).collect::<Vec<DebugVert>>()
                    ]} else { vec![vec![DebugVert{ pos: [0.0;3] }]] },
                    enabled_models: vec![(0, None)],
                )
            ]
        },
        "probes" = {
            material: {
                pipeline: {
                    fragment_shader_path: "shaders/tempFrag.wgsl",
                    vertex_shader_path: "shaders/tempVert.wgsl",
                    uses_camera: false,
                    // temp
                    vertex_layouts: [DisplayVert::vertex_layout()],
                    geometry_details: {
                        topology: wgpu::PrimitiveTopology::LineList,
                        polygon_mode: wgpu::PolygonMode::Line,
                    },
                },
                ident: "ignore"
            },
            computes: [
                Compute(
                    attachments: vec![
                        ShaderAttachment::Buffer(ShaderBufferAttachment::new(
                            device, bytemuck::cast_slice(&[cascade_input]), wgpu::BufferBindingType::Uniform, wgpu::ShaderStages::COMPUTE, wgpu::BufferUsages::empty()
                        )),
                        ShaderAttachment::Buffer(ShaderBufferAttachment::new(
                            device, bytemuck::cast_slice(&surface_objects), wgpu::BufferBindingType::Uniform, wgpu::ShaderStages::COMPUTE, wgpu::BufferUsages::empty()
                        )),
                        // fix this later
                        // order of 'output's follow order of textures passed in
                        ShaderAttachment::Texture(ShaderTextureAttachment { texture_bundle: cascade_texture_bundle.clone(), visibility: wgpu::ShaderStages::COMPUTE }),
                        ShaderAttachment::Texture(ShaderTextureAttachment { texture_bundle: cascade_texture2_bundle.clone(), visibility: wgpu::ShaderStages::COMPUTE }),
                        ShaderAttachment::Texture(ShaderTextureAttachment { texture_bundle: cascade_texture3_bundle.clone(), visibility: wgpu::ShaderStages::COMPUTE }),
                    ],
                    // output: ShaderAttachment::Texture(ShaderTextureAttachment { texture: cascade_texture3.clone(), visibility: wgpu::ShaderStages::COMPUTE, extra_usages: wgpu::TextureUsages::empty() }),
                    shader_path: "shaders/cascadeComputeCalculation.wgsl",
                    workgroup_counts: v4::ecs::compute::WorkgroupCounts::Static(HIGHEST_PROBE_CNT, HIGHEST_PROBE_CNT, 3),
                    ident: "cascade_compute",
                ),
                Compute(
                    attachments: vec![
                        ShaderAttachment::Buffer(ShaderBufferAttachment::new(
                            device, bytemuck::cast_slice(&[merging_input]), wgpu::BufferBindingType::Storage { read_only: false }, wgpu::ShaderStages::COMPUTE, wgpu::BufferUsages::empty()
                        )),
                        ShaderAttachment::Texture(ShaderTextureAttachment { texture_bundle: cascade_texture_merge_bundle, visibility: wgpu::ShaderStages::COMPUTE }),
                        ShaderAttachment::Texture(ShaderTextureAttachment { texture_bundle: cascade_texture2_merge_bundle, visibility: wgpu::ShaderStages::COMPUTE }),
                        ShaderAttachment::Texture(ShaderTextureAttachment { texture_bundle: cascade_texture3_merge_bundle, visibility: wgpu::ShaderStages::COMPUTE }),
                    ],
                    // output: ShaderAttachment::Texture(ShaderTextureAttachment { texture: cascade_texture3_merge, visibility: wgpu::ShaderStages::COMPUTE, extra_usages: wgpu::TextureUsages::empty() }),
                    shader_path: "shaders/cascadeMergeCompute.wgsl",
                    iterate_count: 2,
                    workgroup_counts: v4::ecs::compute::WorkgroupCounts::Static(HIGHEST_PROBE_CNT, HIGHEST_PROBE_CNT, 1),
                    ident: "cascade_merge",
                )
            ],
            components: [
                ComputeTextureTransferComponent(
                    compute_id: ident("cascade_compute"),
                    merge_id: ident("cascade_merge"),
                    ignore_material: ident("ignore"), texture_slot: 0
                )
            ]
        },
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

    fn from_data(data: VertexData) -> Self {
        Self {
            pos: data.pos.try_into().unwrap(),
            tex_coords: data.tex_coords.try_into().unwrap(),
        }
    }
}

/// vertices for drawing debug lines for probe rays
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct DebugVert {
    pos: [f32; 3],
}

impl VertexDescriptor for DebugVert {
    const ATTRIBUTES: &[wgpu::VertexAttribute] = &wgpu::vertex_attr_array![0 => Float32x3,];

    fn from_data(data: VertexData) -> Self {
        Self {
            pos: data.pos.try_into().unwrap(),
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

/// objects in the scene, either surfaces or light sources
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SurfaceObject {
    pub pos: [f32; 2],
    pub radius: f32,
    pub object_type: i32, // 0 for surface, 1 for light
    pub color: [f32; 3],
    pub data: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct MergingInput {
    current_level: u32,
    l0_probe_count: u32,
    l0_angular_sample_count: u32,
}
