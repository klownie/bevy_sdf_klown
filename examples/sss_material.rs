use bevy::core_pipeline::prepass::DepthPrepass;
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use bevy_sdf_klown::engine::{
    camera::RayMarchCamera,
    op::SdOp,
    shape::{SdMaterial, SdMod, SdShape},
};
use bevy_sdf_klown::{RayMarchingPlugin, patients};

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
    commands.spawn((
        SdOp::SmoothUnion {
            pad: [0; 3],
            k: 1.5,
        },
        patients![
            (
                SdShape::Box {
                    bounds: Vec3::new(10.0, 5.0, 10.0),
                },
                Transform::from_xyz(0.0, -2.5, 0.0),
                SdMaterial {
                    color: Vec4::new(0.3, 0.5, 0.3, 1.0),
                    roughness: 0.5,
                    ..default()
                },
            ),
            (
                SdShape::Box {
                    bounds: Vec3::new(2.0, 4.0, 2.0)
                },
                SdMod::CheapBend { k: 0.3 },
                Transform::from_xyz(0.0, 1.9, 0.0),
                SdMaterial {
                    color: Vec4::new(0.7, 0.1, 0.5, 1.0),
                    roughness: 1.0,
                    sss_strength: 0.9,
                    sss_radius: Vec3::new(1.0, 0.7, 0.2),
                    ..default()
                },
            )
        ],
    ));

    // Light
    commands.spawn((
        PointLight {
            intensity: 10000000.0,
            range: 1000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(10.0, 15.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        PointLight {
            intensity: 5000000.0,
            color: Color::LinearRgba(LinearRgba::new(0.5, 0.5, 1.0, 1.0)),
            range: 1000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(0.0, 15.0, -10.0).looking_at(Vec3::ZERO, Vec3::Y),
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
