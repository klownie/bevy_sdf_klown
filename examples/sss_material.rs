use bevy::core_pipeline::prepass::DepthPrepass;
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use bevy_sdf_klown::engine::{
    camera::RayMarchCamera,
    object::{SdMaterial, SdMod, SdShape},
    op::SdOp,
};
use bevy_sdf_klown::{RayMarchingPlugin, op_patients};

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
        .add_systems(Update, animate_twist_modifier)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        SdOp::SmoothUnion {
            _pad: [0; 3],
            k: 1.5,
        },
        op_patients![
            (
                SdShape::Box {
                    bounds: Vec3::new(10.0, 5.0, 10.0),
                },
                SdMod::Twist { k: 0.1 },
                AnimateTwitModifier,
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
struct AnimateTwitModifier;

fn animate_twist_modifier(
    mut query: Query<&mut SdMod, With<AnimateTwitModifier>>,
    time: Res<Time>,
) {
    for mut shape in query.iter_mut() {
        if let SdMod::Twist { k, .. } = &mut *shape {
            let eas_func = EaseFunction::SmootherStep;
            *k = eas_func.sample_unchecked(time.elapsed_secs().sin().abs())
                * time.elapsed_secs_wrapped().sin().signum()
                / 10.0;
        }
    }
}
