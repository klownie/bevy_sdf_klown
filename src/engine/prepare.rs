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
    SdIndex,
    buffer::RayMarchBuffer,
    camera::RayMarchCamera,
    hierarchy::{SdOperatedBy, SdOperatingOn},
    nodes::RayMarchEngineBindGroup,
    object::{SdMaterial, SdModStack, SdModUniform, SdObjectUniform, SdShape, SdTransform},
    op::{SdBlend, SdOperator, SdOperatorUniform},
    pipeline::RayMarchEnginePipeline,
    prepass::RayMarchPrepass,
};

pub fn prepare_raymarch_textures(
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

pub fn prepare_raymarch_bind_group(
    mut commands: Commands,
    query: Query<(&ViewPrepassTextures, &RayMarchPrepass), With<RayMarchCamera>>,
    ray_march_pipeline: Res<RayMarchEnginePipeline>,
    device: Res<RenderDevice>,
    raymarch_buffer: Res<RayMarchBuffer>,
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

    let storage_bind_group = device.create_bind_group(
        "marcher_storage_bind_group",
        &ray_march_pipeline.storage_layout,
        &BindGroupEntries::sequential((
            raymarch_buffer.object.as_entire_binding(),
            raymarch_buffer.operator.as_entire_binding(),
            raymarch_buffer.modifier.as_entire_binding(),
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

pub fn prepare_raymarch_buffer(
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

    let mut sd_object_buffer = BufferVec::<SdObjectUniform>::new(BufferUsages::STORAGE);
    let mut sd_op_buffer = BufferVec::<SdOperatorUniform>::new(BufferUsages::STORAGE);
    let mut sd_mod_buffer = BufferVec::<SdModUniform>::new(BufferUsages::STORAGE);

    let mut push_object = |entity: Entity| -> Option<u16> {
        let (&shape, modifiers, transform, some_mat_handle, some_sd_mat) =
            sdf_object_query.get(entity).ok()?;

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
        let start_index = current_mod_index;
        for &modifier in modifiers.modifiers.iter().rev() {
            current_mod_index = sd_mod_buffer.push(modifier.uniform()) + 1;
        }

        sd_object_buffer.push(SdObjectUniform {
            shape: shape.uniform(),
            material: material.uniform(),
            modifier_stack: modifiers.clone().uniform(start_index),
            transform: transform.uniform(),
        });

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

    sd_object_buffer.write_buffer(&device, &queue);
    sd_op_buffer.write_buffer(&device, &queue);
    sd_mod_buffer.write_buffer(&device, &queue);

    commands.insert_resource(RayMarchBuffer {
        object: sd_object_buffer.buffer().unwrap().clone(),
        operator: sd_op_buffer.buffer().unwrap().clone(),
        modifier: sd_mod_buffer.buffer().unwrap().clone(),
    });
}
