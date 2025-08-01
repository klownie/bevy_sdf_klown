use bevy::core_pipeline::prepass::DepthPrepass;
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use bevy_sdf_klown::engine::object::SdMod;
use bevy_sdf_klown::engine::{
    camera::RayMarchCamera,
    object::{SdMaterial, SdShape},
    op::SdOp,
};
use bevy_sdf_klown::{RayMarchingPlugin, op_patients};

fn main() {
    App::new()
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
        .run();
}

fn setup(mut commands: Commands, mut materials: ResMut<Assets<StandardMaterial>>) {
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

    // Raymarched Sene
    commands.spawn((
        SdOp::SmoothUnion {
            k: 1.0,
            _pad: [0; 3],
        },
        op_patients![
            (
                SdShape::Box {
                    bounds: Vec3::new(10.0, 0.1, 10.0),
                },
                SdMaterial {
                    color: Vec4::new(0.3, 0.5, 0.3, 1.0),
                    roughness: 0.5,
                    ..default()
                },
                SdMod::InfArray {
                    c: Vec3::new(12.0, 10000.0, 12.0),
                },
            ),
            (
                SdOp::Union,
                op_patients![
                    (
                        SdShape::Box {
                            bounds: Vec3::new(1.0, 1.0, 1.0),
                        },
                        Transform::from_xyz(-0.5, 0.5, 0.0),
                        MeshMaterial3d(materials.add(Color::srgb(1., 1., 1.))),
                    ),
                    (
                        SdShape::VerticalCapsule {
                            height: 1.5,
                            radius: 1.0,
                        },
                        SdMod::InfArray {
                            c: Vec3::new(5.0, 1000.0, 4.0),
                        },
                        Transform::from_xyz(2.0, 0.5, 0.0),
                        MeshMaterial3d(materials.add(Color::srgb(0.5, 0.5, 1.))),
                    )
                ]
            )
        ],
    ));

    // Light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 5.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Camera
    commands.spawn((
        RayMarchCamera::default(),
        Camera3d::default(),
        Camera {
            hdr: false,
            msaa_writeback: false,
            ..default()
        },
        PanOrbitCamera::default(),
        Msaa::Off,
        DepthPrepass::default(),
        Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
