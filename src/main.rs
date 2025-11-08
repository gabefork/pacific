use v4::{engine_support::texture_support::Texture, scene, V4};
use wgpu::{StorageTextureAccess, TextureFormat};

#[tokio::main]
async fn main() {
    let mut engine = V4::builder().window_settings(800, 800, "Pacific", None).build().await;

    let rendering_manager = engine.rendering_manager();
    let device = rendering_manager.device();

    let temp_texture = Texture::create_texture(device, 800, 800, TextureFormat::Rgba8Unorm, Some(StorageTextureAccess::ReadOnly), false);

    scene! {
        scene: main_scene,
        active_camera: _,
        screen_space_materials: [
            {
                pipeline: {
                    fragment_shader_path: "",
                },
                attachments: [
                    Texture(
                        texture: temp_texture,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                    )
                ]
            },
        ],
    };

    engine.attach_scene(main_scene);

    engine.main_loop().await;
}
