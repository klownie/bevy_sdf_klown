use bevy::core_pipeline::prepass::DepthPrepass;
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use bevy_sdf_klown::RayMarchingPlugin;
use bevy_sdf_klown::engine::{
    camera::RayMarchCamera,
    op::{SdOp, SdOperatedBy},
    shape::{SdMaterial, SdShape},
};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0)))
        .add_plugins((
            DefaultPlugins,
            RayMarchingPlugin,
            EguiPlugin {
                enable_multipass_for_primary_context: true,
            },
            WorldInspectorPlugin::new(),
            PanOrbitCameraPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, animate_mandelbulb)
        .run();
}

fn setup(mut commands: Commands) {
    // RayMarch Scene
    //
    // This example scene can be illustarted like this:
    //
    // SmoothUnion/
    // ├── PlaneShape
    // └── Union/
    //    ├── BoxShape
    //    └── CapsuleShape

    // WARN: A SdOp will only take in acount TOW RelationShips to it using SdOperatedBy
    // any amount SdOperatedBy used above or under that can BREAK the raymarcher in unexpected ways

    let union1 = commands.spawn((SdOp::Union,)).id();

    let union2 = commands.spawn((SdOp::Union, SdOperatedBy(union1))).id();

    commands.spawn((
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
        SdOperatedBy(union2),
    ));
    commands.spawn((
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
        SdOperatedBy(union2),
    ));
    commands.spawn((
        SdShape::JuliaQuaternion {
            scale: 5.0,
            iter: 30.0,
        },
        Transform::from_xyz(10.0, 0.5, 0.0),
        SdOperatedBy(union1),
        SdMaterial {
            color: Vec4::new(0.9, 0.5, 1.0, 1.0),
            roughness: 0.5,
            ..default()
        },
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
            hdr: false,
            msaa_writeback: false,
            ..default()
        },
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
