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
    merge_id: ComponentId,
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

        let merge = computes
            .iter()
            .filter(|compute| compute.id() == self.merge_id)
            .next();
        if merge.is_none() {
            return;
        }
        let merge = merge.unwrap();

        let material = materials
            .iter()
            .filter(|material| material.id() != self.ignore_material)
            .next();
        if material.is_none() {
            return;
        }
        let material = material.unwrap();
        let compute_output = compute.input_attachments();
        // if let Some(compute_output) = compute.input_attachments() {
            let merge_input = merge.input_attachments();
            for i in 2..5_usize {
                if let ShaderAttachment::Texture(comp_tex) = &compute_output[i] {
                    if let ShaderAttachment::Texture(merge_tex) = &merge_input[i - 1] {
                        encoder.copy_texture_to_texture(
                            comp_tex.texture.texture().as_image_copy(),
                            merge_tex.texture.texture().as_image_copy(),
                            merge_tex.texture.texture().size(),
                        );
                    }
                }
            }
            if let Some(ShaderAttachment::Texture(compute_tex)) = merge_input.get(1) {
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
        // }
    }
}
