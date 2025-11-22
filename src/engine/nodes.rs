use bevy::ecs::query::QueryItem;
use bevy::prelude::*;
use bevy::render::camera::ExtractedCamera;
use bevy::render::render_graph::NodeRunError;
use bevy::render::render_resource::{BindGroup, ComputePassDescriptor, PipelineCache};
use bevy::render::renderer::RenderContext;
use bevy::{
    ecs::system::lifetimeless::Read,
    pbr::ViewLightsUniformOffset,
    render::{
        extract_component::DynamicUniformIndex,
        render_graph::{RenderGraphContext, ViewNode},
        view::{ViewTarget, ViewUniformOffset},
    },
};

use super::pipeline::RayMarchEnginePipeline;
use super::{RayMarchCamera, WORKGROUP_SIZE};

#[derive(Resource)]
pub struct RayMarchEngineBindGroup {
    pub common_bind_group: BindGroup,
    pub texture_bind_group: BindGroup,
    pub storage_bind_group: BindGroup,
    pub prepass_bind_group: BindGroup,
}

#[derive(Default)]
pub struct RayMarchEngineNode;

impl ViewNode for RayMarchEngineNode {
    type ViewQuery = (
        Read<ViewTarget>,
        Read<ExtractedCamera>,
        Read<RayMarchCamera>,
        Read<DynamicUniformIndex<RayMarchCamera>>,
        Read<ViewUniformOffset>,
        Read<ViewLightsUniformOffset>,
    );

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (
            _view_target,
            camera,
            ray_march_settings,
            settings_index,
            view_uniform_offset,
            view_lights_uniform_offset,
        ): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        if !world.contains_resource::<RayMarchEngineBindGroup>() {
            return Ok(());
        }

        let pipeline_cache = world.resource::<PipelineCache>();
        let ray_march_pipeline = world.resource::<RayMarchEnginePipeline>();
        let bind_group = world.resource::<RayMarchEngineBindGroup>();

        let (Some(march_pipeline), Some(scale_pipeline)) = (
            pipeline_cache.get_compute_pipeline(ray_march_pipeline.compute_raymarch_pipeline),
            pipeline_cache.get_compute_pipeline(ray_march_pipeline.compute_mask_pipeline),
        ) else {
            return Ok(());
        };

        let Some(viewport) = camera.physical_viewport_size else {
            return Ok(());
        };

        let mut pass =
            render_context
                .command_encoder()
                .begin_compute_pass(&ComputePassDescriptor {
                    label: Some("ray_march_pass"),
                    timestamp_writes: None,
                });

        pass.set_bind_group(
            0,
            &bind_group.common_bind_group,
            &[
                view_uniform_offset.offset,
                view_lights_uniform_offset.offset,
            ],
        );
        pass.set_bind_group(1, &bind_group.texture_bind_group, &[settings_index.index()]);
        pass.set_bind_group(2, &bind_group.storage_bind_group, &[]);
        pass.set_bind_group(3, &bind_group.prepass_bind_group, &[]);

        pass.set_pipeline(march_pipeline);
        pass.dispatch_workgroups(
            ((viewport.x as f32 * ray_march_settings.depth_scale) as u32).div_ceil(WORKGROUP_SIZE),
            ((viewport.y as f32 * ray_march_settings.depth_scale) as u32).div_ceil(WORKGROUP_SIZE),
            1,
        );

        pass.set_pipeline(scale_pipeline);
        pass.dispatch_workgroups(
            viewport.x.div_ceil(WORKGROUP_SIZE),
            viewport.y.div_ceil(WORKGROUP_SIZE),
            1,
        );

        Ok(())
    }
}
