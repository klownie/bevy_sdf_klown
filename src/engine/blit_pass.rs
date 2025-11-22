use bevy::{
    core_pipeline::{
        FullscreenShader, core_3d::CORE_3D_DEPTH_FORMAT, prepass::ViewPrepassTextures,
    },
    ecs::{query::QueryItem, system::lifetimeless::Read},
    prelude::*,
    render::{
        render_graph::{NodeRunError, RenderGraphContext, ViewNode},
        render_resource::{
            BindGroupEntries, BindGroupLayout, BindGroupLayoutEntries, CachedRenderPipelineId,
            ColorTargetState, ColorWrites, CompareFunction, DepthBiasState, DepthStencilState,
            FragmentState, LoadOp, Operations, PipelineCache, RenderPassColorAttachment,
            RenderPassDescriptor, RenderPipelineDescriptor, Sampler, SamplerBindingType,
            SamplerDescriptor, ShaderStages, StencilState, StoreOp, TextureSampleType,
            binding_types::{sampler, texture_2d},
        },
        renderer::{RenderContext, RenderDevice},
        texture::DepthAttachment,
        view::ViewTarget,
    },
};

use crate::engine::{RAY_MARCH_BLIT_PASS_HANDLE, camera::RayMarchCamera, prepass::RayMarchPrepass};

#[derive(Default)]
pub struct BlitNode;

impl ViewNode for BlitNode {
    type ViewQuery = (
        Read<ViewTarget>,
        Read<ViewPrepassTextures>,
        Read<RayMarchCamera>,
        Read<RayMarchPrepass>,
    );

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (view_target, view_prepass_textures, _post_process_settings, raymarch_prepass): QueryItem<
            Self::ViewQuery,
        >,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let raymarch_blit_pipeline = world.resource::<RayMarchPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        let Some(pipeline) = pipeline_cache.get_render_pipeline(raymarch_blit_pipeline.pipeline_id)
        else {
            return Ok(());
        };

        let bind_group = render_context.render_device().create_bind_group(
            "raymarch_blit_bind_group",
            &raymarch_blit_pipeline.layout,
            &BindGroupEntries::sequential((
                &raymarch_prepass.depth,
                &raymarch_prepass.output,
                &raymarch_blit_pipeline.sampler,
            )),
        );

        let depth = DepthAttachment::new(view_prepass_textures.depth_view().unwrap().clone(), None);
        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("raymarch_blit_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: view_target.get_unsampled_color_attachment().view,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(depth.get_attachment(StoreOp::Store)),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_render_pipeline(pipeline);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}

#[derive(Resource)]
struct RayMarchPipeline {
    layout: BindGroupLayout,
    sampler: Sampler,
    pipeline_id: CachedRenderPipelineId,
}

pub(crate) fn init_raymarch_blit_pipeline(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    fullscreen_shader: Res<FullscreenShader>,
    pipeline_cache: Res<PipelineCache>,
) {
    let layout = render_device.create_bind_group_layout(
        "post_process_bind_group_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                texture_2d(TextureSampleType::Float { filterable: true }),
                texture_2d(TextureSampleType::Float { filterable: true }),
                sampler(SamplerBindingType::Filtering),
            ),
        ),
    );
    let sampler = render_device.create_sampler(&SamplerDescriptor::default());

    let vertex_state = fullscreen_shader.to_vertex_state();
    let pipeline_id = pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
        label: Some("raymarch_blit_pipeline".into()),
        layout: vec![layout.clone()],
        vertex: vertex_state,
        fragment: Some(FragmentState {
            shader: RAY_MARCH_BLIT_PASS_HANDLE,
            targets: vec![Some(ColorTargetState {
                format: ViewTarget::TEXTURE_FORMAT_HDR,
                blend: None,
                write_mask: ColorWrites::ALL,
            })],
            ..default()
        }),
        depth_stencil: Some(DepthStencilState {
            format: CORE_3D_DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: CompareFunction::Greater,
            stencil: StencilState::default(),
            bias: DepthBiasState::default(),
        }),
        ..default()
    });
    commands.insert_resource(RayMarchPipeline {
        layout,
        sampler,
        pipeline_id,
    });
}
