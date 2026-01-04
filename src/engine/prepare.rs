use bevy::{
    core_pipeline::prepass::ViewPrepassTextures,
    pbr::{GlobalClusterableObjectMeta, LightMeta},
    prelude::*,
    render::{
        camera::ExtractedCamera,
        extract_component::ComponentUniforms,
        render_resource::{
            BindGroupEntries, BufferUsages, BufferVec, Extent3d, PipelineCache, TextureAspect,
            TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
            TextureViewDescriptor,
        },
        renderer::{RenderDevice, RenderQueue},
        view::ViewUniforms,
    },
};

use crate::engine::{
    SdIndex,
    buffer::RayMarchBuffer,
    camera::RayMarchCamera,
    hierarchy::{SdOperatedBy, SdOperatingOn},
    nodes::RayMarchEngineBindGroup,
    object::{
        SdMaterial, SdModStack, SdModUniform, SdObject, SdObjectUniform, SdShape, SdTransform,
    },
    op::{SdBlend, SdOperator, SdOperatorUniform},
    pipeline::RayMarchEnginePipeline,
    prepass::RayMarchPrepass,
};

pub(crate) fn prepare_raymarch_textures(
    query: Query<(
        Entity,
        &ExtractedCamera,
        Option<&RayMarchPrepass>,
        &RayMarchCamera,
    )>,
    render_device: Res<RenderDevice>,
    mut commands: Commands,
) {
    for (entity, camera, ray_march_resources, ray_march_settings) in &query {
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

        let scaled_size = Extent3d {
            width: (view_size.x as f32 * ray_march_settings.depth_scale) as u32,
            height: (view_size.y as f32 * ray_march_settings.depth_scale) as u32,
            depth_or_array_layers: 1,
        };

        let depth = render_device
            .create_texture(&TextureDescriptor {
                label: Some("raymarch_depth_texture"),
                size: scaled_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::R32Float,
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
                label: Some("raymarch_normal_texture"),
                size: scaled_size,
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

        let mask = render_device
            .create_texture(&TextureDescriptor {
                label: Some("raymarch_mask_texture"),
                size: size,
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

        let output = render_device
            .create_texture(&TextureDescriptor {
                label: Some("raymarch_material_texture"),
                size: scaled_size,
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

        commands.entity(entity).insert(RayMarchPrepass {
            depth,
            normal,
            mask,
            output,
            view_size,
        });
    }
}

pub(crate) fn prepare_raymarch_bind_group(
    mut commands: Commands,
    device: Res<RenderDevice>,
    query: Query<(&ViewPrepassTextures, &RayMarchPrepass), With<RayMarchCamera>>,
    ray_march_pipeline: Res<RayMarchEnginePipeline>,
    raymarch_buffer: Option<Res<RayMarchBuffer>>,
    pipeline_cache: Res<PipelineCache>,
    settings_uniforms: Res<ComponentUniforms<RayMarchCamera>>,
    view_uniforms: Res<ViewUniforms>,
    light_meta: Res<LightMeta>,
    clusterables: Res<GlobalClusterableObjectMeta>,
) {
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

    let march_buffer = unsafe { raymarch_buffer.unwrap_unchecked() };

    let texture_bind_group = device.create_bind_group(
        "ray_march_texture_bind_group",
        &pipeline_cache.get_bind_group_layout(&ray_march_pipeline.texture_layout),
        &BindGroupEntries::sequential((
            view_prepass.depth_view().unwrap(),
            settings_binding.clone(),
        )),
    );

    let common_bind_group = device.create_bind_group(
        "ray_march_view_bind_group",
        &pipeline_cache.get_bind_group_layout(&ray_march_pipeline.common_layout),
        &BindGroupEntries::with_indices((
            (0, view_binding.clone()),
            (1, light_binding.clone()),
            (8, cluster_binding.clone()),
        )),
    );

    let storage_bind_group = device.create_bind_group(
        "marcher_storage_bind_group",
        &pipeline_cache.get_bind_group_layout(&ray_march_pipeline.storage_layout),
        &BindGroupEntries::sequential((
            march_buffer.object.as_entire_buffer_binding(),
            march_buffer.operator.as_entire_buffer_binding(),
            march_buffer.modifier.as_entire_buffer_binding(),
            march_buffer.field_data.as_entire_buffer_binding(),
        )),
    );

    let prepass_bind_group = device.create_bind_group(
        "marcher_prepass_bind_group",
        &pipeline_cache.get_bind_group_layout(&ray_march_pipeline.prepass_layout),
        &BindGroupEntries::sequential((
            &raymarch_prepass.depth,
            &raymarch_prepass.normal,
            &raymarch_prepass.mask,
            &raymarch_prepass.output,
        )),
    );

    commands.insert_resource(RayMarchEngineBindGroup {
        common_bind_group,
        texture_bind_group,
        storage_bind_group,
        prepass_bind_group,
    });
}

pub(crate) fn prepare_raymarch_buffer(
    mut commands: Commands,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    sdf_object_query: Query<
        (
            &SdShape,
            &SdModStack,
            &GlobalTransform,
            Option<&MeshMaterial3d<StandardMaterial>>,
            Option<&SdMaterial>,
        ),
        With<SdOperatedBy>,
    >,
    sd_op_query: Query<(&SdBlend, &SdIndex, &SdOperatingOn)>,
    material_as: Res<Assets<StandardMaterial>>,
) {
    let nb_shapes = sdf_object_query.iter().len() as u16;

    let mut current_shape_index = 0;
    let mut current_op_index = 0;
    let mut current_mod_index = 0;
    let mut current_field_data_index = 0;

    let mut sd_object_buffer = BufferVec::<SdObjectUniform>::new(BufferUsages::STORAGE);
    let mut sd_op_buffer = BufferVec::<SdOperatorUniform>::new(BufferUsages::STORAGE);
    let mut sd_mod_buffer = BufferVec::<SdModUniform>::new(BufferUsages::STORAGE);
    let mut sd_field_data_buffer = BufferVec::<f32>::new(BufferUsages::STORAGE);

    let mut push_object = |entity: Entity| -> Option<u16> {
        let (&shape, modifier_stack, transform, some_mat_handle, some_sd_mat) =
            sdf_object_query.get(entity).ok()?;

        let nb_shape_field = shape.gpu_field_count();
        let start_field_data_index = current_field_data_index;
        current_field_data_index += nb_shape_field;

        for &field in shape.flatten_fields().iter() {
            sd_field_data_buffer.push(field / 2.); // div by 2 is for matching with the blender
        }

        let material = match (some_mat_handle, some_sd_mat) {
            (Some(mat_handle), _) => {
                let std_material = material_as.get(mat_handle.id()).unwrap_or_else(|| {
                    panic!("Material handle found but not available in Assets<StandardMaterial>")
                });
                SdMaterial::from(std_material.clone())
            }
            (None, Some(sd_mat)) => *sd_mat,
            (None, None) => {
                panic!(
                    "Entity {:?} is missing both MeshMaterial3d and SdMaterial",
                    entity
                );
            }
        };

        let transform = SdTransform {
            pos: transform.translation(),
            rot: Vec3::from(transform.rotation().to_euler(EulerRot::XYZ)),
        };

        // Push modifiers and count them
        let start_mod_index = current_mod_index;
        for &modifier in modifier_stack.modifiers.iter().rev() {
            current_mod_index = sd_mod_buffer.push(modifier.uniform()) + 1;
        }

        sd_object_buffer.push(
            SdObject {
                shape,
                material,
                modifier_stack: modifier_stack.clone(),
                transform,
            }
            .uniform(start_mod_index, start_field_data_index),
        );

        let i = Some(current_shape_index);
        current_shape_index += 1;
        i
    };

    for (&op, _index, op_on) in sd_op_query.iter().sort_unstable::<&SdIndex>().rev() {
        let mut compute_index = |patient: Entity| -> Option<u16> {
            if sd_op_query.get(patient).is_ok() {
                let i = Some(nb_shapes + current_op_index);
                current_op_index += 1;
                i
            } else {
                push_object(patient)
            }
        };

        let args = op_on.clone().get_sd_argunments();
        let lhs = compute_index(args.1).unwrap_or(0);
        let rhs = compute_index(args.0).unwrap_or(0);

        sd_op_buffer.push(SdOperator { op, lhs, rhs }.uniform());
    }

    current_mod_index
        .eq(&0)
        .then(|| sd_mod_buffer.push(SdModUniform::default()));

    sd_object_buffer.write_buffer(&device, &queue);
    sd_op_buffer.write_buffer(&device, &queue);
    sd_mod_buffer.write_buffer(&device, &queue);
    sd_field_data_buffer.write_buffer(&device, &queue);

    if let (Some(object_buf), Some(operator_buf), Some(modifier_buf), Some(field_data_buf)) = (
        sd_object_buffer.buffer(),
        sd_op_buffer.buffer(),
        sd_mod_buffer.buffer(),
        sd_field_data_buffer.buffer(),
    ) {
        commands.insert_resource(RayMarchBuffer {
            object: object_buf.clone(),
            operator: operator_buf.clone(),
            modifier: modifier_buf.clone(),
            field_data: field_data_buf.clone(),
        });
    }
}
