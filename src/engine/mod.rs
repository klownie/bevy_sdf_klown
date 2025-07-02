use bevy::asset::{RenderAssetUsages, load_internal_asset, weak_handle};
use bevy::ecs::component::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use bevy::render::extract_component::ExtractComponent;
use bevy::render::extract_resource::{ExtractResource, ExtractResourcePlugin};
use bevy::render::render_graph::RenderLabel;
use bevy::render::texture::GpuImage;
use bevy::render::view::ExtractedView;
use bevy::render::{Render, RenderSet};
use bevy::{
    core_pipeline::core_3d::graph::{Core3d, Node3d},
    render::{
        RenderApp,
        extract_component::{ExtractComponentPlugin, UniformComponentPlugin},
        render_graph::{RenderGraphApp, ViewNodeRunner},
        render_resource::*,
    },
};
use camera::RayMarchCamera;
use nodes::RayMarchEngineNode;
use op::{InitSkeinSdRelatinShip, SdOp, SdOpInstance, SdOperatedBy, SdOperatingOn};
use pipeline::{RayMarchEnginePipeline, RayMarchEnginePipelineKey};
use shape::{SdMaterial, SdMod, SdShape, SdShapeInstance, SdTransform};
use upscale::RayMarchUpscalePlugin;

mod nodes;
mod pipeline;

mod upscale;

pub mod camera;
pub mod op;
pub mod shape;

const RAY_MARCH_MAIN_PASS_HANDLE: Handle<Shader> =
    weak_handle!("ca4a5dbf-4da9-4779-bcdc-dd3186088e08");
const RAY_MARCH_UPSCALE_PASS_HANDLE: Handle<Shader> =
    weak_handle!("a780d707-67bf-45b5-b77e-76dad6c17e5f");
const RAY_MARCH_UTILS_HANDLE: Handle<Shader> = weak_handle!("0a9451d0-4b19-453b-98bc-ec755845d8f3");
const RAY_MARCH_TYPES_HANDLE: Handle<Shader> = weak_handle!("689f31b3-bdf6-4770-b18a-3979d671045c");
const RAY_MARCH_SELECTORS_HANDLE: Handle<Shader> =
    weak_handle!("47df8567-7cf9-49a2-8939-0e81c2aa2f93");

const WORKGROUP_SIZE: u32 = 8;

#[derive(Component, ExtractComponent, Clone)]
pub struct RayMarchPrepass {
    pub depth: Handle<Image>,
    pub normal: Handle<Image>,
    pub material: Handle<Image>,
    pub shadow: Handle<Image>,
    pub mask: Handle<Image>,
}

impl RayMarchPrepass {
    pub fn new(asset_server: &AssetServer) -> Self {
        let mut r_image = Image::new(
            Extent3d {
                width: 3840,
                height: 2160,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            vec![0; 33177600],
            TextureFormat::R32Float,
            RenderAssetUsages::RENDER_WORLD,
        );
        r_image.texture_descriptor.usage = TextureUsages::COPY_DST
            | TextureUsages::STORAGE_BINDING
            | TextureUsages::TEXTURE_BINDING;

        let depth = asset_server.add(r_image.clone());
        let shadow = asset_server.add(r_image.clone());
        let mask = asset_server.add(r_image);

        let mut rgb_image = Image::new(
            Extent3d {
                width: 3840,
                height: 2160,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            vec![0; 132710400],
            TextureFormat::Rgba32Float,
            RenderAssetUsages::RENDER_WORLD,
        );
        rgb_image.texture_descriptor.usage = TextureUsages::COPY_DST
            | TextureUsages::STORAGE_BINDING
            | TextureUsages::TEXTURE_BINDING;

        let normal = asset_server.add(rgb_image.clone());
        let material = asset_server.add(rgb_image);

        Self {
            depth,
            normal,
            material,
            shadow,
            mask,
        }
    }
}

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
            ExtractComponentPlugin::<RayMarchPrepass>::default(),
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
            .init_resource::<SpecializedComputePipelines<RayMarchEnginePipeline>>()
            .add_render_graph_node::<ViewNodeRunner<RayMarchEngineNode>>(
                Core3d,
                RayMarchPass::MarchPass,
            )
            .add_render_graph_edges(
                Core3d,
                (
                    Node3d::EndMainPass,
                    RayMarchPass::MarchPass,
                    RayMarchPass::MainPass,
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
pub struct RayMarchEnginePipelineId(pub CachedComputePipelineId);

fn prepare_ray_march_pipelines(
    mut commands: Commands,
    pipeline_cache: Res<PipelineCache>,
    mut pipelines: ResMut<SpecializedComputePipelines<RayMarchEnginePipeline>>,
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
    MainPass,
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

    let mut push_shape = |entity: Entity| -> Option<u16> {
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
}

#[derive(Resource, Reflect, Default, Clone, ExtractResource)]
#[reflect(Resource, Default)]
pub struct SdShapeStorage {
    pub data: Vec<SdShapeInstance>,
}

#[derive(Resource, Reflect, Debug, Default, Clone, ExtractResource)]
#[reflect(Resource, Default)]
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
