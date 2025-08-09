use bevy::{
    core_pipeline::prepass::ViewPrepassTextures,
    pbr::{GlobalClusterableObjectMeta, LightMeta},
    prelude::*,
    render::{
        camera::ExtractedCamera,
        extract_component::ComponentUniforms,
        render_resource::{
            BindGroupEntries, BufferUsages, BufferVec, Extent3d, TextureAspect, TextureDescriptor,
            TextureDimension, TextureFormat, TextureUsages, TextureViewDescriptor,
        },
        renderer::{RenderDevice, RenderQueue},
        view::ViewUniforms,
    },
};

use crate::engine::{
    SdObjectStorage, SdOperatorStorage,
    camera::RayMarchCamera,
    nodes::RayMarchEngineBindGroup,
    object::{SdModUniform, SdObjectUniform},
    op::SdOperatorUniform,
    pipeline::RayMarchEnginePipeline,
    prepass::RayMarchPrepass,
};

pub fn prepare_ray_march_resources(
    query: Query<(Entity, &ExtractedCamera, Option<&RayMarchPrepass>), With<RayMarchCamera>>,
    render_device: Res<RenderDevice>,
    mut commands: Commands,
) {
    for (entity, camera, ray_march_resources) in &query {
        let Some(view_size) = camera.physical_viewport_size else {
            continue;
        };

        if ray_march_resources.map(|r| r.view_size) == Some(view_size) {
            continue;
        }

        let size = Extent3d {
            width: view_size.x,
            height: view_size.y,
            depth_or_array_layers: 1,
        };

        let depth = render_device
            .create_texture(&TextureDescriptor {
                label: Some("raymarch_depth"),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::R16Float,
                usage: TextureUsages::COPY_DST
                    | TextureUsages::STORAGE_BINDING
                    | TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            })
            .create_view(&TextureViewDescriptor {
                label: Some("raymarch_depth_view"),
                base_mip_level: 0,
                aspect: TextureAspect::All,
                base_array_layer: 0,
                ..default()
            });

        let normal = render_device
            .create_texture(&TextureDescriptor {
                label: Some("raymarch_normal"),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba16Float,
                usage: TextureUsages::COPY_DST
                    | TextureUsages::STORAGE_BINDING
                    | TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            })
            .create_view(&TextureViewDescriptor {
                label: Some("raymarch_normal_view"),
                base_mip_level: 0,
                aspect: TextureAspect::All,
                base_array_layer: 0,
                ..default()
            });

        let material = render_device
            .create_texture(&TextureDescriptor {
                label: Some("raymarch_material"),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba16Float,
                usage: TextureUsages::COPY_DST
                    | TextureUsages::STORAGE_BINDING
                    | TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            })
            .create_view(&TextureViewDescriptor {
                label: Some("raymarch_material_view"),
                base_mip_level: 0,
                aspect: TextureAspect::All,
                base_array_layer: 0,
                ..default()
            });

        let mask = render_device
            .create_texture(&TextureDescriptor {
                label: Some("raymarch_mask"),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::R16Float,
                usage: TextureUsages::COPY_DST
                    | TextureUsages::STORAGE_BINDING
                    | TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            })
            .create_view(&TextureViewDescriptor {
                label: Some("raymarch_mask_view"),
                base_mip_level: 0,
                aspect: TextureAspect::All,
                base_array_layer: 0,
                ..default()
            });

        let scaled_depth = render_device
            .create_texture(&TextureDescriptor {
                label: Some("scaled_raymarch_depth"),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::R16Float,
                usage: TextureUsages::COPY_DST
                    | TextureUsages::STORAGE_BINDING
                    | TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            })
            .create_view(&TextureViewDescriptor {
                label: Some("scaled_raymarch_depth_view"),
                base_mip_level: 0,
                aspect: TextureAspect::All,
                base_array_layer: 0,
                ..default()
            });

        let scaled_normal = render_device
            .create_texture(&TextureDescriptor {
                label: Some("scaled_raymarch_normal"),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba16Float,
                usage: TextureUsages::COPY_DST
                    | TextureUsages::STORAGE_BINDING
                    | TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            })
            .create_view(&TextureViewDescriptor {
                label: Some("scaled_raymarch_normal_view"),
                base_mip_level: 0,
                aspect: TextureAspect::All,
                base_array_layer: 0,
                ..default()
            });

        let scaled_material = render_device
            .create_texture(&TextureDescriptor {
                label: Some("scaled_raymarch_material"),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba16Float,
                usage: TextureUsages::COPY_DST
                    | TextureUsages::STORAGE_BINDING
                    | TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            })
            .create_view(&TextureViewDescriptor {
                label: Some("scaled_raymarch_material_view"),
                base_mip_level: 0,
                aspect: TextureAspect::All,
                base_array_layer: 0,
                ..default()
            });

        let scaled_mask = render_device
            .create_texture(&TextureDescriptor {
                label: Some("scaled_raymarch_mask"),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::R16Float,
                usage: TextureUsages::COPY_DST
                    | TextureUsages::STORAGE_BINDING
                    | TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            })
            .create_view(&TextureViewDescriptor {
                label: Some("scaled_raymarch_mask_view"),
                base_mip_level: 0,
                aspect: TextureAspect::All,
                base_array_layer: 0,
                ..default()
            });

        commands.entity(entity).insert(RayMarchPrepass {
            depth,
            normal,
            material,
            mask,
            scaled_depth,
            scaled_normal,
            scaled_material,
            scaled_mask,
            view_size,
        });
    }
}

pub fn prepare_ray_march_bind_group(
    mut commands: Commands,
    query: Query<(&ViewPrepassTextures, &RayMarchPrepass), With<RayMarchCamera>>,
    ray_march_pipeline: Res<RayMarchEnginePipeline>,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    settings_uniforms: Res<ComponentUniforms<RayMarchCamera>>,
    view_uniforms: Res<ViewUniforms>,
    light_meta: Res<LightMeta>,
    clusterables: Res<GlobalClusterableObjectMeta>,
    object_storage: Res<SdObjectStorage>,
    operator_storage: Res<SdOperatorStorage>,
) {
    log::info!("preparing_bindgroup");
    let Ok((view_prepass, raymarch_prepass)) = query.single() else {
        return;
    };
    let (Some(settings_binding), Some(view_binding), Some(light_binding), Some(cluster_binding)) = (
        settings_uniforms.uniforms().binding(),
        view_uniforms.uniforms.binding(),
        light_meta.view_gpu_lights.binding(),
        clusterables.gpu_clusterable_objects.binding(),
    ) else {
        return;
    };

    let texture_bind_group = device.create_bind_group(
        "ray_march_texture_bind_group",
        &ray_march_pipeline.texture_layout,
        &BindGroupEntries::sequential((
            // view_target.get_unsampled_color_attachment().view,
            view_prepass.depth_view().unwrap(),
            settings_binding.clone(),
        )),
    );

    let common_bind_group = device.create_bind_group(
        "ray_march_view_bind_group",
        &ray_march_pipeline.common_layout,
        &BindGroupEntries::with_indices((
            (0, view_binding.clone()),
            (1, light_binding.clone()),
            (8, cluster_binding.clone()),
        )),
    );

    let mut sd_mod_buf = BufferVec::<SdModUniform>::new(BufferUsages::STORAGE);
    // let mut sd_mod_data_buf = BufferVec::<f32>::new(BufferUsages::STORAGE);
    let mut sd_object_buf = BufferVec::<SdObjectUniform>::new(BufferUsages::STORAGE);
    sd_object_buf.reserve(object_storage.data.len(), &device);

    let mut current_mod_index = 0;
    object_storage.data.iter().for_each(|obj| {
        // Push modifiers and count them
        let start_index = current_mod_index;
        for &modifier in obj.modifier_stack.modifiers.iter().rev() {
            current_mod_index = sd_mod_buf.push(modifier.uniform()) + 1;
        }

        sd_object_buf.push(SdObjectUniform {
            shape: obj.shape.clone().uniform(),
            material: obj.material.uniform(),
            modifier_stack: obj.modifier_stack.clone().uniform(start_index),
            transform: obj.transform.uniform(),
        });
    });

    sd_mod_buf.reserve(current_mod_index, &device);

    let mut sd_op_buf = BufferVec::<SdOperatorUniform>::new(BufferUsages::STORAGE);
    sd_op_buf.reserve(operator_storage.data.len(), &device);
    operator_storage.data.iter().for_each(|&op| {
        sd_op_buf.push(op.uniform());
    });

    sd_mod_buf
        .is_empty()
        .then(|| sd_mod_buf.push(SdModUniform::default()));

    sd_object_buf.write_buffer(&device, &queue);
    sd_op_buf.write_buffer(&device, &queue);
    sd_mod_buf.write_buffer(&device, &queue);

    let storage_bind_group = device.create_bind_group(
        "marcher_storage_bind_group",
        &ray_march_pipeline.storage_layout,
        &BindGroupEntries::sequential((
            sd_object_buf.binding().unwrap(),
            sd_op_buf.binding().unwrap(),
            sd_mod_buf.binding().unwrap(),
        )),
    );

    let prepass_bind_group = device.create_bind_group(
        "marcher_prepass_bind_group",
        &ray_march_pipeline.prepass_layout,
        &BindGroupEntries::sequential((
            &raymarch_prepass.depth,
            &raymarch_prepass.normal,
            &raymarch_prepass.material,
            &raymarch_prepass.mask,
            &raymarch_prepass.scaled_depth,
            &raymarch_prepass.scaled_normal,
            &raymarch_prepass.scaled_material,
            &raymarch_prepass.scaled_mask,
        )),
    );

    commands.insert_resource(RayMarchEngineBindGroup {
        common_bind_group,
        texture_bind_group,
        storage_bind_group,
        prepass_bind_group,
    });
}
