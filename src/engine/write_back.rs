use bevy::{
    core_pipeline::{
        core_3d::graph::Core3d, fullscreen_vertex_shader::fullscreen_shader_vertex_state,
    },
    ecs::{query::QueryItem, system::lifetimeless::Read},
    prelude::*,
    render::{
        Render, RenderApp, RenderSet,
        extract_component::{ComponentUniforms, DynamicUniformIndex},
        render_graph::{
            NodeRunError, RenderGraphApp, RenderGraphContext, ViewNode, ViewNodeRunner,
        },
        render_resource::{
            binding_types::{sampler, texture_2d, uniform_buffer},
            *,
        },
        renderer::{RenderContext, RenderDevice},
        view::{ExtractedView, ViewTarget},
    },
};

use super::{MARCH_WRITE_BACK_PASS_HANDLE, RayMarchCamera, RayMarchPass, RayMarchPrepass};

pub struct MarchWriteBackPlugin;

impl Plugin for MarchWriteBackPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(());

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .add_systems(
                Render,
                prepare_ray_march_pipelines.in_set(RenderSet::Prepare),
            )
            .init_resource::<SpecializedRenderPipelines<MarchWriteBackPipeline>>()
            .add_render_graph_node::<ViewNodeRunner<MarchWriteBackNode>>(
                Core3d,
                RayMarchPass::WriteBackPass,
            );
    }

    fn finish(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.init_resource::<MarchWriteBackPipeline>();
    }
}

#[derive(Component, Deref, DerefMut)]
pub struct MarchWriteBackPipelineId(pub CachedRenderPipelineId);

fn prepare_ray_march_pipelines(
    mut commands: Commands,
    pipeline_cache: Res<PipelineCache>,
    mut pipelines: ResMut<SpecializedRenderPipelines<MarchWriteBackPipeline>>,
    post_processing_pipeline: Res<MarchWriteBackPipeline>,
    views: Query<(Entity, &ExtractedView), With<RayMarchCamera>>,
) {
    for (entity, view) in views.iter() {
        let pipeline_id = pipelines.specialize(
            &pipeline_cache,
            &post_processing_pipeline,
            UpscalePipelineKey { hdr: view.hdr },
        );

        commands
            .entity(entity)
            .insert(MarchWriteBackPipelineId(pipeline_id));
    }
}

#[derive(Default)]
struct MarchWriteBackNode;

impl ViewNode for MarchWriteBackNode {
    type ViewQuery = (
        Read<ViewTarget>,
        Read<RayMarchCamera>,
        Read<DynamicUniformIndex<RayMarchCamera>>,
        Read<MarchWriteBackPipelineId>,
        Read<RayMarchPrepass>,
    );

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (view_target, _pixelate_settings, settings_index, pipeline_id, raymarch_prepass): QueryItem<
            Self::ViewQuery,
        >,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let post_process_pipeline = world.resource::<MarchWriteBackPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        let Some(pipeline) = pipeline_cache.get_render_pipeline(**pipeline_id) else {
            return Ok(());
        };

        // Get the settings uniform binding
        let settings_uniforms = world.resource::<ComponentUniforms<RayMarchCamera>>();
        let Some(settings_binding) = settings_uniforms.uniforms().binding() else {
            return Ok(());
        };

        let post_process = view_target.post_process_write();

        let bind_group = render_context.render_device().create_bind_group(
            "march_write_back_pass_bind_group",
            &post_process_pipeline.layout,
            &BindGroupEntries::sequential((
                post_process.source,
                &post_process_pipeline.sampler,
                settings_binding.clone(),
            )),
        );

        let prepass_bind_group = render_context.render_device().create_bind_group(
            "marcher_prepass_bind_group",
            &post_process_pipeline.prepass_layout,
            &BindGroupEntries::sequential((&raymarch_prepass.material, &raymarch_prepass.mask)),
        );

        // Begin the render pass
        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("march_write_back_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: post_process.destination,
                resolve_target: None,
                ops: Operations::default(),
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_render_pipeline(pipeline);
        render_pass.set_bind_group(0, &bind_group, &[settings_index.index()]);
        render_pass.set_bind_group(1, &prepass_bind_group, &[]);
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}

#[derive(Resource)]
struct MarchWriteBackPipeline {
    layout: BindGroupLayout,
    sampler: Sampler,
    prepass_layout: BindGroupLayout,
}

impl FromWorld for MarchWriteBackPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let layout = render_device.create_bind_group_layout(
            "march_write_back_bind_group_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    sampler(SamplerBindingType::Filtering),
                    uniform_buffer::<RayMarchCamera>(true),
                ),
            ),
        );

        let sampler = render_device.create_sampler(&SamplerDescriptor::default());

        let prepass_layout = render_device.create_bind_group_layout(
            "march_write_back_prepass_bind_group_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    texture_2d(TextureSampleType::Float { filterable: true }),
                ),
            ),
        );

        Self {
            layout,
            sampler,
            prepass_layout,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct UpscalePipelineKey {
    pub hdr: bool,
}

impl SpecializedRenderPipeline for MarchWriteBackPipeline {
    type Key = UpscalePipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        let format = if key.hdr {
            ViewTarget::TEXTURE_FORMAT_HDR
        } else {
            TextureFormat::bevy_default()
        };

        RenderPipelineDescriptor {
            label: Some("march_write_back_pipeline".into()),
            layout: vec![self.layout.clone(), self.prepass_layout.clone()],
            vertex: fullscreen_shader_vertex_state(),
            fragment: Some(FragmentState {
                shader: MARCH_WRITE_BACK_PASS_HANDLE,
                shader_defs: vec![],
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            push_constant_ranges: vec![],
            zero_initialize_workgroup_memory: false,
        }
    }
}
