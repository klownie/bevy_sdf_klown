use bevy::prelude::*;
use bevy::render::render_resource::TextureUsages;
use bevy::render::view::Hdr;
use bevy::{camera::CameraMainTextureUsages, core_pipeline::prepass::DepthPrepass};
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use bevy_sdf_klown::engine::{
    camera::RayMarchCamera,
    object::{SdMaterial, SdShape},
    op::SdBlend,
};
use bevy_sdf_klown::{RayMarchingPlugin, op_patients};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0)))
        .add_plugins((
            DefaultPlugins,
            RayMarchingPlugin,
            EguiPlugin::default(),
            WorldInspectorPlugin::new(),
            PanOrbitCameraPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, animate_mandelbulb)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        SdBlend::Union,
        op_patients![
            (
                SdBlend::Union,
                op_patients![
                    (
                        SdShape::MengerSponge {
                            scale: 5.0,
                            iter: 10.0,
                        },
                        SdMaterial {
                            color: Vec4::new(0.9, 0.5, 1.0, 1.0),
                            roughness: 0.5,
                            ..default()
                        },
                        Transform::from_xyz(-10.0, 0.5, 0.0),
                    ),
                    (
                        AnimateMadelBulb,
                        SdShape::MandelBulb {
                            scale: 5.0,
                            expo: 8.0,
                            iter: 10.0,
                            b_offset: 0.0,
                        },
                        SdMaterial {
                            color: Vec4::new(0.9, 0.5, 1.0, 1.0),
                            roughness: 0.5,
                            ..default()
                        },
                        Transform::from_xyz(0.0, 0.5, 0.0),
                    )
                ]
            ),
            (
                SdShape::JuliaQuaternion {
                    scale: 5.0,
                    iter: 30.0,
                },
                Transform::from_xyz(10.0, 0.5, 0.0),
                SdMaterial {
                    color: Vec4::new(0.9, 0.5, 1.0, 1.0),
                    roughness: 0.5,
                    ..default()
                },
            )
        ],
    ));

    // Light
    commands.spawn((
        PointLight {
            intensity: 20000000.0,
            range: 1000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(0.0, 15.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        PointLight {
            intensity: 20000000.0,
            color: Color::LinearRgba(LinearRgba::new(0.5, 0.5, 1.0, 1.0)),
            range: 1000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(0.0, -15.0, -10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Camera
    commands.spawn((
        RayMarchCamera {
            depth_scale: 0.4,
            ..default()
        },
        Camera3d::default(),
        Camera {
            msaa_writeback: false,
            ..default()
        },
        Hdr,
        CameraMainTextureUsages::default().with(TextureUsages::STORAGE_BINDING),
        PanOrbitCamera::default(),
        Msaa::Off,
        DepthPrepass::default(),
        Transform::from_xyz(-0.0, 5.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

#[derive(Component)]
struct AnimateMadelBulb;

fn animate_mandelbulb(mut query: Query<&mut SdShape, With<AnimateMadelBulb>>) {
    for mut shape in query.iter_mut() {
        if let SdShape::MandelBulb { b_offset, .. } = &mut *shape {
            *b_offset += 0.01;
        }
    }
}
