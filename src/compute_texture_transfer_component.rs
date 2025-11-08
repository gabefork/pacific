use v4::{
    component,
    ecs::{
        component::{ComponentDetails, ComponentId, ComponentSystem},
        material::ShaderAttachment,
    },
};

#[component]
pub struct ComputeTextureTransferComponent {
    compute_id: ComponentId,
    ignore_material: ComponentId,
    texture_slot: usize,
}

impl ComponentSystem for ComputeTextureTransferComponent {
    fn command_encoder_operations(
        &self,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        _other_components: &[&v4::ecs::component::Component],
        materials: &[v4::ecs::material::Material],
        computes: &[v4::ecs::compute::Compute],
    ) {
        let compute = computes
            .iter()
            .filter(|compute| compute.id() == self.compute_id)
            .next();
        if compute.is_none() {
            return;
        }
        let compute = compute.unwrap();

        let material = materials
            .iter()
            .filter(|material| material.id() != self.ignore_material)
            .next();
        if material.is_none() {
            return;
        }
        let material = material.unwrap();
        if let Some(compute_output) = compute.output_attachments() {
            if let ShaderAttachment::Texture(compute_tex) = compute_output {
                if let Some(ShaderAttachment::Texture(material_tex)) =
                    material.attachments().get(self.texture_slot)
                {
                    encoder.copy_texture_to_texture(
                        compute_tex.texture.texture().as_image_copy(),
                        material_tex.texture.texture().as_image_copy(),
                        compute_tex.texture.texture().size(),
                    );
                }
            }
        }
    }
}
