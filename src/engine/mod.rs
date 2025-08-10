use bevy::asset::{load_internal_asset, weak_handle};
use bevy::prelude::*;
use bevy::render::extract_resource::ExtractResourcePlugin;
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
use hierarchy::{SdOperatedBy, SdOperatingOn};
use nodes::RayMarchEngineNode;
use object::{SdMaterial, SdMod, SdShape};
use op::SdBlend;
use pipeline::RayMarchEnginePipeline;
use write_back::MarchWriteBackPlugin;

use crate::engine::buffer::RayMarchBuffer;
use crate::engine::object::SdModStack;
use crate::engine::op::SdIndex;
use crate::engine::prepare::{
    prepare_raymarch_bind_group, prepare_raymarch_buffer, prepare_raymarch_textures,
};

#[cfg(feature = "skein")]
use hierarchy::InitSkeinSdRelationShip;

mod nodes;
mod pipeline;

mod write_back;

pub mod buffer;
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

        app.add_systems(
            Update,
            (prepare_raymarch_buffer.run_if(
                not(resource_exists::<RayMarchBuffer>)
                    .or(ray_march_object_buffer_needs_update)
                    .or(ray_march_operator_buffer_needs_update),
            ),),
        );

        app.add_plugins((
            ExtractComponentPlugin::<RayMarchCamera>::default(),
            UniformComponentPlugin::<RayMarchCamera>::default(),
            ExtractResourcePlugin::<RayMarchBuffer>::default(),
        ))
        .add_plugins(MarchWriteBackPlugin)
        .register_type::<RayMarchCamera>()
        .register_type::<SdShape>()
        .register_type::<SdBlend>()
        .register_type::<SdMod>()
        .register_type::<SdModStack>()
        .register_type::<SdIndex>()
        .register_type::<SdMaterial>();

        #[cfg(feature = "skein")]
        app.register_type::<InitSkeinSdRelationShip>();

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .add_systems(
                Render,
                (
                    prepare_raymarch_textures.in_set(RenderSet::PrepareResources),
                    prepare_raymarch_bind_group.in_set(RenderSet::PrepareBindGroups), // .run_if(not(resource_exists::<RayMarchEngineBindGroup>)),
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
