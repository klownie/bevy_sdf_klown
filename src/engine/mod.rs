use bevy::asset::{load_internal_asset, uuid_handle};
use bevy::prelude::*;
use bevy::render::extract_resource::ExtractResourcePlugin;
use bevy::render::render_graph::RenderLabel;
use bevy::render::{Render, RenderStartup, RenderSystems};
use bevy::shader::load_shader_library;
use bevy::{
    core_pipeline::core_3d::graph::{Core3d, Node3d},
    render::{
        RenderApp,
        extract_component::{ExtractComponentPlugin, UniformComponentPlugin},
        render_graph::ViewNodeRunner,
    },
};
use camera::RayMarchCamera;
use hierarchy::{SdOperatedBy, SdOperatingOn};
use nodes::RayMarchEngineNode;
use object::{SdMaterial, SdMod, SdShape};
use op::SdBlend;

use crate::engine::blit_pass::{BlitNode, init_raymarch_blit_pipeline};
use crate::engine::buffer::RayMarchBuffer;
use crate::engine::object::SdModStack;
use crate::engine::op::SdIndex;
use crate::engine::pipeline::init_raymarch_compute_pipeline;
use crate::engine::prepare::{
    prepare_raymarch_bind_group, prepare_raymarch_buffer, prepare_raymarch_textures,
};
use bevy::render::render_graph::RenderGraphExt;

#[cfg(feature = "skein")]
use hierarchy::InitSkeinSdRelationShip;

mod blit_pass;
mod nodes;
mod pipeline;

pub mod buffer;
pub mod camera;
pub mod hierarchy;
pub mod object;
pub mod op;
pub mod prepare;
pub mod prepass;

const RAY_MARCH_COMPUTE_PASS_HANDLE: Handle<Shader> =
    uuid_handle!("ca4a5dbf-4da9-4779-bcdc-dd3186088e08");
const RAY_MARCH_BLIT_PASS_HANDLE: Handle<Shader> =
    uuid_handle!("691fef51-0ad4-4131-81c6-f71e674505ab");

const WORKGROUP_SIZE: u32 = 8;

pub struct RayMarchEnginePlugin;

impl Plugin for RayMarchEnginePlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            RAY_MARCH_COMPUTE_PASS_HANDLE,
            "../shaders/ray_march.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            RAY_MARCH_BLIT_PASS_HANDLE,
            "../shaders/blit.wgsl",
            Shader::from_wgsl
        );

        load_shader_library!(app, "../shaders/bindings.wgsl");
        load_shader_library!(app, "../shaders/utils.wgsl");
        load_shader_library!(app, "../shaders/types.wgsl");
        load_shader_library!(app, "../shaders/selectors.wgsl");

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
            .add_systems(RenderStartup, init_raymarch_compute_pipeline)
            .add_systems(
                Render,
                (
                    prepare_raymarch_textures
                        .in_set(RenderSystems::PrepareAssets)
                        .run_if(resource_exists::<RayMarchBuffer>),
                    prepare_raymarch_bind_group
                        .in_set(RenderSystems::PrepareBindGroups)
                        .run_if(resource_exists::<RayMarchBuffer>),
                ),
            )
            .add_render_graph_node::<ViewNodeRunner<RayMarchEngineNode>>(
                Core3d,
                RayMarchPass::ComputePass,
            )
            .add_render_graph_edges(Core3d, (Node3d::EndMainPass, RayMarchPass::ComputePass));

        render_app
            .add_systems(RenderStartup, init_raymarch_blit_pipeline)
            .add_render_graph_node::<ViewNodeRunner<BlitNode>>(Core3d, RayMarchPass::BlitPass)
            .add_render_graph_edges(Core3d, (RayMarchPass::ComputePass, RayMarchPass::BlitPass));
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub enum RayMarchPass {
    ComputePass,
    BlitPass,
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
