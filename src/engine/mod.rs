use bevy::asset::{load_internal_asset, weak_handle};
use bevy::core_pipeline::prepass::ViewPrepassTextures;
use bevy::ecs::component::HookContext;
use bevy::ecs::system::lifetimeless::Read;
use bevy::ecs::world::DeferredWorld;
use bevy::pbr::{
    GlobalClusterableObjectMeta, GpuClusterableObjectsStorage, GpuLights, LightMeta,
    ViewLightsUniformOffset,
};
use bevy::prelude::*;
use bevy::render::{Render, RenderSet};
use log::warn;
use op::{
    InitSkeinSdRelatinShip, SdOp, SdOpInstance, SdOpUniform, SdOpUniformInstance, SdOperatedBy,
    SdOperatingOn,
};
use shape::{
    SdMaterial, SdMaterialUniform, SdMod, SdShape, SdShapeInstance, SdShapeUniform,
    SdShapeUniformInstance, SdTransform, SdTransformUniform,
};

use bevy::render::extract_resource::{ExtractResource, ExtractResourcePlugin};
use bevy::render::render_resource::binding_types::{storage_buffer_read_only, texture_depth_2d};
use bevy::render::renderer::RenderQueue;
use bevy::render::view::{ExtractedView, ViewUniform, ViewUniformOffset, ViewUniforms};
use bevy::{
    core_pipeline::{
        core_3d::graph::{Core3d, Node3d},
        fullscreen_vertex_shader::fullscreen_shader_vertex_state,
    },
    ecs::query::QueryItem,
    render::{
        RenderApp,
        extract_component::{
            ComponentUniforms, DynamicUniformIndex, ExtractComponent, ExtractComponentPlugin,
            UniformComponentPlugin,
        },
        render_graph::{
            NodeRunError, RenderGraphApp, RenderGraphContext, RenderLabel, ViewNode, ViewNodeRunner,
        },
        render_resource::{
            binding_types::{sampler, texture_2d, uniform_buffer},
            *,
        },
        renderer::{RenderContext, RenderDevice},
        view::ViewTarget,
    },
};
use upscale::RayMarchUpscalePlugin;

mod op;
mod shape;
mod upscale;

const RAY_MARCH_MAIN_PASS_HANDLE: Handle<Shader> =
    weak_handle!("ca4a5dbf-4da9-4779-bcdc-dd3186088e08");
const RAY_MARCH_UPSCALE_PASS_HANDLE: Handle<Shader> =
    weak_handle!("a780d707-67bf-45b5-b77e-76dad6c17e5f");

const RAY_MARCH_UTILS_HANDLE: Handle<Shader> = weak_handle!("0a9451d0-4b19-453b-98bc-ec755845d8f3");
const RAY_MARCH_TYPES_HANDLE: Handle<Shader> = weak_handle!("689f31b3-bdf6-4770-b18a-3979d671045c");
const RAY_MARCH_SELECTORS_HANDLE: Handle<Shader> =
    weak_handle!("47df8567-7cf9-49a2-8939-0e81c2aa2f93");

pub struct RayMarchEnginePlugin;

impl Plugin for RayMarchEnginePlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            RAY_MARCH_MAIN_PASS_HANDLE,
            "../../assets/ray_marching.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            RAY_MARCH_UPSCALE_PASS_HANDLE,
            "../../assets/upscale.wgsl",
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
            ExtractResourcePlugin::<SdShapeStorage>::default(),
            ExtractResourcePlugin::<SdOpStorage>::default(),
        ))
        .add_systems(PostUpdate, update_ray_march_buffer)
        .add_plugins(RayMarchUpscalePlugin)
        .register_type::<RayMarchCamera>()
        .register_type::<SdShape>()
        .register_type::<SdOp>()
        .register_type::<SdMod>()
        .register_type::<SdIndex>()
        .register_type::<InitSkeinSdRelatinShip>()
        .register_type::<SdMaterial>()
        .init_resource::<SdShapeStorage>()
        .init_resource::<SdOpStorage>();

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .add_systems(
                Render,
                prepare_ray_march_pipelines.in_set(RenderSet::Prepare),
            )
            .init_resource::<SpecializedRenderPipelines<RayMarchEnginePipeline>>()
            .add_render_graph_node::<ViewNodeRunner<RayMarchEngineNode>>(
                Core3d,
                RayMarchPass::MarchPass,
            )
            .add_render_graph_edges(
                Core3d,
                (
                    Node3d::EndMainPass,
                    RayMarchPass::MarchPass,
                    RayMarchPass::UpscalePass,
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

#[derive(Component, Deref, DerefMut)]
pub struct RayMarchEnginePipelineId(pub CachedRenderPipelineId);

fn prepare_ray_march_pipelines(
    mut commands: Commands,
    pipeline_cache: Res<PipelineCache>,
    mut pipelines: ResMut<SpecializedRenderPipelines<RayMarchEnginePipeline>>,
    post_processing_pipeline: Res<RayMarchEnginePipeline>,
    views: Query<(Entity, &ExtractedView), With<RayMarchCamera>>,
) {
    for (entity, view) in views.iter() {
        let pipeline_id = pipelines.specialize(
            &pipeline_cache,
            &post_processing_pipeline,
            RayMarchEnginePipelineKey { hdr: view.hdr },
        );

        commands
            .entity(entity)
            .insert(RayMarchEnginePipelineId(pipeline_id));
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub enum RayMarchPass {
    MarchPass,
    UpscalePass,
}

#[derive(Default)]
struct RayMarchEngineNode;

impl ViewNode for RayMarchEngineNode {
    type ViewQuery = (
        Read<ViewTarget>,
        Read<RayMarchCamera>,
        Read<DynamicUniformIndex<RayMarchCamera>>,
        Read<ViewPrepassTextures>,
        Read<ViewUniformOffset>,
        Read<ViewLightsUniformOffset>,
        Read<RayMarchEnginePipelineId>,
    );

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (
            view_target,
            _ray_march_settings,
            settings_index,
            view_prepass,
            view_uniform_offset,
            view_lights_uniform_offset,
            pipeline_id,
        ): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let ray_march_pipeline = world.resource::<RayMarchEnginePipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        let Some(pipeline) = pipeline_cache.get_render_pipeline(**pipeline_id) else {
            return Ok(());
        };

        let settings_uniforms = world.resource::<ComponentUniforms<RayMarchCamera>>();
        let Some(settings_binding) = settings_uniforms.uniforms().binding() else {
            return Ok(());
        };

        let post_process = view_target.post_process_write();

        let (Some(depth_prepass), Some(normal_prepass), Some(motion_prepass)) = (
            view_prepass.depth_view(),
            view_prepass.normal_view(),
            view_prepass.motion_vectors_view(),
        ) else {
            return Ok(());
        };

        let texture_bind_group = render_context.render_device().create_bind_group(
            "ray_march_texture_bind_group",
            &ray_march_pipeline.texture_layout,
            &BindGroupEntries::sequential((
                post_process.source,
                &ray_march_pipeline.sampler,
                depth_prepass,
                settings_binding.clone(),
            )),
        );

        let view_uniforms = world.get_resource::<ViewUniforms>().unwrap();
        let Some(view_binding) = view_uniforms.uniforms.binding() else {
            return Ok(());
        };

        let light = world.get_resource::<LightMeta>().unwrap();
        let Some(light_binding) = light.view_gpu_lights.binding() else {
            return Ok(());
        };

        let clusterable_objects = world.get_resource::<GlobalClusterableObjectMeta>().unwrap();
        let Some(clusterables_objects_binding) =
            clusterable_objects.gpu_clusterable_objects.binding()
        else {
            return Ok(());
        };

        let common_bind_group = render_context.render_device().create_bind_group(
            "ray_march_view_bind_group",
            &ray_march_pipeline.common_layout,
            &BindGroupEntries::with_indices((
                (0, view_binding.clone()),
                (1, light_binding.clone()),
                (8, clusterables_objects_binding.clone()),
            )),
        );

        let sd_shape_ressource = world.resource::<SdShapeStorage>();
        let mut sd_shape_storage = BufferVec::<SdShapeUniformInstance>::new(BufferUsages::STORAGE);
        for &sd_shape in sd_shape_ressource.data.iter() {
            sd_shape_storage.push(SdShapeUniformInstance {
                shape: sd_shape.shape.uniform(),
                material: sd_shape.material.uniform(),
                modifier: sd_shape.modifier.uniform(),
                transform: sd_shape.transform.uniform(),
            });
        }

        let sd_op_ressource = world.resource::<SdOpStorage>();
        let mut sd_op_storage = BufferVec::<SdOpUniformInstance>::new(BufferUsages::STORAGE);
        for &sd_op in sd_op_ressource.data.iter() {
            sd_op_storage.push(sd_op.uniform());
        }

        let render_device = world.resource::<RenderDevice>();
        let render_queue = world.resource::<RenderQueue>();
        sd_shape_storage.write_buffer(render_device, render_queue);
        sd_op_storage.write_buffer(render_device, render_queue);

        let storage_bind_group = render_context.render_device().create_bind_group(
            "marcher_storage_bind_group",
            &ray_march_pipeline.storage_layout,
            &BindGroupEntries::sequential((
                sd_shape_storage.binding().unwrap(),
                sd_op_storage.binding().unwrap(),
            )),
        );

        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("ray_march_pass"),
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
        render_pass.set_bind_group(
            0,
            &common_bind_group,
            &[
                view_uniform_offset.offset,
                view_lights_uniform_offset.offset,
            ],
        );
        render_pass.set_bind_group(1, &texture_bind_group, &[settings_index.index()]);
        render_pass.set_bind_group(2, &storage_bind_group, &[]);
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}

#[derive(Resource)]
struct RayMarchEnginePipeline {
    common_layout: BindGroupLayout,
    texture_layout: BindGroupLayout,
    storage_layout: BindGroupLayout,
    sampler: Sampler,
}

impl FromWorld for RayMarchEnginePipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let common_layout = render_device.create_bind_group_layout(
            "ray_march_import_bind_group_layout",
            &BindGroupLayoutEntries::with_indices(
                ShaderStages::FRAGMENT,
                (
                    (0, uniform_buffer::<ViewUniform>(true)),
                    // Directional Lights
                    (1, uniform_buffer::<GpuLights>(true)),
                    // Spotlights
                    (
                        8,
                        storage_buffer_read_only::<GpuClusterableObjectsStorage>(false),
                    ),
                ),
            ),
        );

        let texture_layout = render_device.create_bind_group_layout(
            "ray_march_texture_bind_group_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    sampler(SamplerBindingType::Filtering),
                    texture_depth_2d(),
                    uniform_buffer::<RayMarchCamera>(true),
                ),
            ),
        );

        let storage_layout = render_device.create_bind_group_layout(
            "ray_march_storage_bind_group_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    storage_buffer_read_only::<SdShapeUniformInstance>(false),
                    storage_buffer_read_only::<SdOpUniformInstance>(false),
                ),
            ),
        );

        let sampler = render_device.create_sampler(&SamplerDescriptor::default());

        Self {
            common_layout,
            texture_layout,
            storage_layout,
            sampler,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct RayMarchEnginePipelineKey {
    pub hdr: bool,
}

impl SpecializedRenderPipeline for RayMarchEnginePipeline {
    type Key = RayMarchEnginePipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        let format = if key.hdr {
            ViewTarget::TEXTURE_FORMAT_HDR
        } else {
            TextureFormat::bevy_default()
        };

        RenderPipelineDescriptor {
            label: Some("ray_march_pipeline".into()),
            layout: vec![
                self.common_layout.clone(),
                self.texture_layout.clone(),
                self.storage_layout.clone(),
            ],
            vertex: fullscreen_shader_vertex_state(),
            fragment: Some(FragmentState {
                shader: RAY_MARCH_MAIN_PASS_HANDLE,
                shader_defs: vec![],
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format,
                    blend: None,
                    write_mask: ColorWrites::COLOR,
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

#[derive(Component, Default, Clone, Copy, Reflect, ExtractComponent, ShaderType)]
#[reflect(Component)]
pub struct RayMarchCamera {
    pub downscale: f32,
    pub eps: f32,
    pub max_distance: f32,
    pub max_steps: u32,
    pub shadow_eps: f32,
    pub shadow_max_steps: u32,
    pub shadow_max_distance: f32,
    pub normal_eps: f32,
}

fn update_ray_march_buffer(
    sdf_shape_query: Query<
        (
            &SdShape,
            &SdMod,
            &GlobalTransform,
            &MeshMaterial3d<StandardMaterial>,
            Option<&SdMaterial>,
        ),
        With<SdOperatedBy>,
    >,
    mut sd_shape_buffer: ResMut<SdShapeStorage>,
    sdf_op_query: Query<(&SdOp, &SdIndex, &SdOperatingOn)>,
    mut sd_op_buffer: ResMut<SdOpStorage>,
    material_as: Res<Assets<StandardMaterial>>,
) {
    let nb_shapes = sdf_shape_query.iter().len() as u16;
    let mut current_shape_index = 0;
    let mut current_op_index = 0;

    sd_shape_buffer.data = Vec::with_capacity(nb_shapes as usize);
    sd_op_buffer.data = Vec::with_capacity(sdf_op_query.iter().len());

    let mut push_shape = #[inline]
    |entity: Entity| -> Option<u16> {
        let (&shape, &modifier, transform, mat_handle, some_sd_mat) =
            sdf_shape_query.get(entity).ok()?;
        let std_material = material_as.get(mat_handle.id())?;

        let transform = SdTransform {
            pos: transform.translation(),
            rot: Vec3::from(transform.rotation().to_euler(EulerRot::XYZ)),
        };

        let &material = some_sd_mat.unwrap_or(&SdMaterial::from(std_material.clone()));

        sd_shape_buffer.data.push(SdShapeInstance {
            shape,
            material,
            modifier,
            transform,
        });

        let i = Some(current_shape_index);
        current_shape_index += 1;
        i
    };

    for (&op, _index, op_on) in sdf_op_query.iter().sort_unstable::<&SdIndex>().rev() {
        let mut compute_index = |patient: Entity| -> Option<u16> {
            if sdf_op_query.get(patient).is_ok() {
                let i = Some(nb_shapes + current_op_index);
                current_op_index += 1;
                i
            } else {
                push_shape(patient)
            }
        };

        let args = op_on.clone().get_sd_argunments();
        let lhs = compute_index(args.1).unwrap_or(0);
        let rhs = compute_index(args.0).unwrap_or(0);

        sd_op_buffer.data.push(SdOpInstance { op, lhs, rhs });
    }

    warn! {"{:#?}", sd_op_buffer};
}

#[derive(Resource, Reflect, Default, Clone, ExtractResource)]
#[reflect(Resource, Default, Clone)]
pub struct SdShapeStorage {
    pub data: Vec<SdShapeInstance>,
}

#[derive(Resource, Reflect, Debug, Default, Clone, ExtractResource)]
#[reflect(Resource, Default, Clone)]
pub struct SdOpStorage {
    pub data: Vec<SdOpInstance>,
}

#[derive(Reflect, Component, Ord, PartialOrd, PartialEq, Eq, Default, Debug, Clone, Copy)]
#[component(on_add = update_sd_index)]
#[reflect(Component)]
pub struct SdIndex(u32);

fn update_sd_index(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    let index = {
        let parent = match world.get::<ChildOf>(entity) {
            Some(p) => p.parent(),
            None => return set_index(&mut world, entity, 0),
        };

        match world.get::<SdIndex>(parent) {
            Some(sd_index) => sd_index.0 + 1,
            None => 0,
        }
    };

    set_index(&mut world, entity, index);
}

#[inline]
fn set_index(world: &mut DeferredWorld, entity: Entity, index: u32) {
    if let Some(mut sd_index) = world.get_mut::<SdIndex>(entity) {
        sd_index.0 = index;
    }
}
