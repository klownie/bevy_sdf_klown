use bevy::asset::{load_internal_asset, weak_handle};
use bevy::core_pipeline::core_3d::prepare_prepass_textures;
use bevy::ecs::component::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use bevy::render::extract_resource::{ExtractResource, ExtractResourcePlugin};
use bevy::render::render_graph::RenderLabel;
use bevy::render::{Render, RenderSet};
use bevy::{
    core_pipeline::core_3d::graph::{Core3d, Node3d},
    render::{
        RenderApp,
        extract_component::{ExtractComponentPlugin, UniformComponentPlugin},
        render_graph::{RenderGraphApp, ViewNodeRunner},
    },
};
use camera::RayMarchCamera;
use hierarchy::{InitSkeinSdRelationShip, SdOperatedBy, SdOperatingOn};
use nodes::RayMarchEngineNode;
use object::{SdMaterial, SdMod, SdObject, SdShape, SdTransform};
use op::{SdBlend, SdOperator};
use pipeline::RayMarchEnginePipeline;
use write_back::MarchWriteBackPlugin;

use crate::engine::nodes::RayMarchEngineBindGroup;
use crate::engine::object::SdModStack;
use crate::engine::prepare::prepare_ray_march_bind_group;

mod nodes;
mod pipeline;

mod write_back;

pub mod camera;
pub mod hierarchy;
pub mod object;
pub mod op;
pub mod prepare;
pub mod prepass;

const RAY_MARCH_MAIN_PASS_HANDLE: Handle<Shader> =
    weak_handle!("ca4a5dbf-4da9-4779-bcdc-dd3186088e08");
const MARCH_WRITE_BACK_PASS_HANDLE: Handle<Shader> =
    weak_handle!("a780d707-67bf-45b5-b77e-76dad6c17e5f");
const RAY_MARCH_UTILS_HANDLE: Handle<Shader> = weak_handle!("0a9451d0-4b19-453b-98bc-ec755845d8f3");
const RAY_MARCH_TYPES_HANDLE: Handle<Shader> = weak_handle!("689f31b3-bdf6-4770-b18a-3979d671045c");
const RAY_MARCH_SELECTORS_HANDLE: Handle<Shader> =
    weak_handle!("47df8567-7cf9-49a2-8939-0e81c2aa2f93");

const WORKGROUP_SIZE: u32 = 8;

pub struct RayMarchEnginePlugin;

impl Plugin for RayMarchEnginePlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            RAY_MARCH_MAIN_PASS_HANDLE,
            "../shaders/ray_march.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            MARCH_WRITE_BACK_PASS_HANDLE,
            "../shaders/write_back.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            RAY_MARCH_UTILS_HANDLE,
            "../shaders/utils.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            RAY_MARCH_TYPES_HANDLE,
            "../shaders/types.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            RAY_MARCH_SELECTORS_HANDLE,
            "../shaders/selectors.wgsl",
            Shader::from_wgsl
        );

        app.add_plugins((
            ExtractComponentPlugin::<RayMarchCamera>::default(),
            UniformComponentPlugin::<RayMarchCamera>::default(),
            ExtractResourcePlugin::<SdObjectStorage>::default(),
            ExtractResourcePlugin::<SdOperatorStorage>::default(),
        ))
        .add_systems(
            PostUpdate,
            (update_ray_march_buffer
                .run_if(
                    ray_march_operator_buffer_needs_update.or(ray_march_object_buffer_needs_update),
                )
                .in_set(RayMarchSet),),
        )
        .add_plugins(MarchWriteBackPlugin)
        .register_type::<RayMarchCamera>()
        .register_type::<SdShape>()
        .register_type::<SdBlend>()
        .register_type::<SdMod>()
        .register_type::<SdModStack>()
        .register_type::<SdIndex>()
        .register_type::<InitSkeinSdRelationShip>()
        .register_type::<SdMaterial>()
        .init_resource::<SdObjectStorage>()
        .init_resource::<SdOperatorStorage>();

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .add_systems(
                Render,
                (
                    prepare_prepass_textures.in_set(RenderSet::PrepareResources),
                    prepare_ray_march_bind_group
                        .in_set(RenderSet::PrepareBindGroups)
                        .run_if(
                            not(resource_exists::<RayMarchEngineBindGroup>)
                                .or(resource_changed::<SdObjectStorage>)
                                .or(resource_changed::<SdOperatorStorage>),
                        ),
                ),
            )
            .add_render_graph_node::<ViewNodeRunner<RayMarchEngineNode>>(
                Core3d,
                RayMarchPass::RayMarchPass,
            )
            .add_render_graph_edges(
                Core3d,
                (
                    Node3d::EndMainPass,
                    RayMarchPass::RayMarchPass,
                    RayMarchPass::WriteBackPass,
                    Node3d::Bloom,
                ),
            );
    }

    fn finish(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render_app.init_resource::<RayMarchEnginePipeline>();
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub enum RayMarchPass {
    RayMarchPass,
    WriteBackPass,
}

fn update_ray_march_buffer(
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
    mut sd_object_buffer: ResMut<SdObjectStorage>,
    sd_op_query: Query<(&SdBlend, &SdIndex, &SdOperatingOn)>,
    mut sd_op_buffer: ResMut<SdOperatorStorage>,
    material_as: Res<Assets<StandardMaterial>>,
) {
    let nb_shapes = sdf_object_query.iter().len() as u16;
    let mut current_shape_index = 0;
    let mut current_op_index = 0;

    sd_object_buffer.data = Vec::with_capacity(nb_shapes as usize);
    sd_op_buffer.data = Vec::with_capacity(sd_op_query.iter().len());

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

        sd_object_buffer.data.push(SdObject {
            shape,
            material,
            modifier_stack: modifiers.clone(),
            transform,
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

        sd_op_buffer.data.push(SdOperator { op, lhs, rhs });
    }
    // log::info!("{:#?}", sd_op_buffer.data);
    // log::info!("objects :{:#?}", sd_object_buffer.data);
}

#[warn(dead_code)]
fn update_object_buffer(
    sdf_object_query: Query<
        (
            Entity,
            &SdShape,
            &SdModStack,
            &GlobalTransform,
            Option<&MeshMaterial3d<StandardMaterial>>,
            Option<&SdMaterial>,
        ),
        With<SdOperatedBy>,
    >,
    mut sd_object_buffer: ResMut<SdObjectStorage>,
    material_as: Res<Assets<StandardMaterial>>,
) {
    let nb_shapes = sdf_object_query.iter().len() as u16;
    sd_object_buffer.data = Vec::with_capacity(nb_shapes as usize);

    for (entity, &shape, modifiers, transform, some_mat_handle, some_sd_mat) in sdf_object_query {
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

        sd_object_buffer.data.push(SdObject {
            shape,
            material,
            modifier_stack: modifiers.clone(),
            transform,
        });
    }

    // log::info!("objects :{:#?}", sd_object_buffer.data);
}

fn ray_march_operator_buffer_needs_update(
    check_op_query: Query<
        (),
        (
            With<SdBlend>,
            With<SdIndex>,
            With<SdOperatingOn>,
            Or<(Changed<SdBlend>, Changed<SdIndex>, Changed<SdOperatingOn>)>,
        ),
    >,
) -> bool {
    !check_op_query.is_empty()
}

fn ray_march_object_buffer_needs_update(
    check_object_query: Query<
        (),
        (
            With<SdShape>,
            With<SdModStack>,
            With<GlobalTransform>,
            With<SdOperatedBy>,
            Or<(
                Changed<SdShape>,
                Changed<SdModStack>,
                Changed<GlobalTransform>,
                Changed<SdMaterial>,
                Changed<MeshMaterial3d<StandardMaterial>>,
            )>,
        ),
    >,
) -> bool {
    !check_object_query.is_empty()
}

#[derive(Resource, Default, Debug, Clone, ExtractResource)]
pub struct SdObjectStorage {
    pub data: Vec<SdObject>,
}

#[derive(Resource, Default, Debug, Clone, ExtractResource)]
pub struct SdOperatorStorage {
    pub data: Vec<SdOperator>,
}

#[derive(Reflect, Component, Ord, PartialOrd, PartialEq, Eq, Default, Debug, Clone, Copy)]
#[component(on_add = update_sd_index)]
#[reflect(Component)]
pub struct SdIndex(pub u32);

fn update_sd_index(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    let mut depth = 0;
    let mut current_entity = entity;

    while let Some(parent) = world.get::<SdOperatedBy>(current_entity) {
        depth += 1;
        current_entity = parent.0;
    }

    set_index(&mut world, entity, depth);
}

#[inline]
fn set_index(world: &mut DeferredWorld, entity: Entity, index: u32) {
    if let Some(mut sd_index) = world.get_mut::<SdIndex>(entity) {
        sd_index.0 = index;
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct RayMarchSet;
