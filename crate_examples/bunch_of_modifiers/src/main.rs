use bevy::camera_controller::free_camera::FreeCamera;
use bevy::prelude::*;
use bevy::render::view::Hdr;
use bevy::{
    camera_controller::free_camera::FreeCameraPlugin, core_pipeline::prepass::DepthPrepass,
};
// use bevy_egui::EguiPlugin;
// use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_sdf_klown::engine::{
    camera::RayMarchCamera,
    object::{SdMaterial, SdMod, SdModStack, SdShape},
    op::SdBlend,
};
use bevy_sdf_klown::{RayMarchingPlugin, op_patients};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0)))
        .add_plugins((
            DefaultPlugins,
            RayMarchingPlugin,
            FreeCameraPlugin,
            // EguiPlugin::default(),
            // WorldInspectorPlugin::new(),
        ))
        .insert_resource(GlobalAmbientLight {
            color: Color::LinearRgba(LinearRgba {
                red: 0.6,
                green: 0.08,
                blue: 0.5,
                alpha: 0.0,
            }),
            ..default()
        })
        .add_systems(Startup, setup)
        .add_systems(Update, animate_twist_modifier)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        SdBlend::SmoothUnion { k: 1.5 },
        op_patients![
            (
                SdShape::RoundBox {
                    bounds: Vec3::new(10.0, 5.0, 5.0),
                    radius: 1.0,
                },
                SdModStack {
                    modifiers: vec![
                        SdMod::InfArray {
                            c: Vec3::new(14.0, 40.0, 9.0)
                        },
                        SdMod::RotateY { a: 0.8 },
                        SdMod::Twist { k: 0.1 },
                        SdMod::CheapBend { k: 0.01 }
                    ]
                },
                AnimateTwitModifier,
                Transform::from_xyz(0.0, -2.5, 0.0),
                SdMaterial {
                    color: LinearRgba::new(0.3, 0.5, 0.3, 1.0).into(),
                    roughness: 0.5,
                    ..default()
                },
            ),
            (
                SdShape::Box {
                    bounds: Vec3::new(2.0, 4.0, 2.0)
                },
                SdModStack {
                    modifiers: vec![
                        SdMod::CheapBend { k: 0.3 },
                        SdMod::Elongate {
                            h: Vec3::new(0.0, 2.0, 0.0)
                        },
                    ]
                },
                Transform::from_xyz(0.0, 1.9, 0.0),
                SdMaterial {
                    color: LinearRgba::new(0.7, 0.1, 0.5, 1.0).into(),
                    roughness: 1.0,
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
        Transform::from_xyz(10.0, 10.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Camera
    commands.spawn((
        RayMarchCamera {
            depth_scale: 0.3,
            ..default()
        },
        Camera3d::default(),
        Camera {
            msaa_writeback: MsaaWriteback::Off,
            ..default()
        },
        FreeCamera::default(),
        Hdr,
        Msaa::Off,
        DepthPrepass::default(),
        Transform::from_xyz(-0.0, 5.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

#[derive(Component)]
struct AnimateTwitModifier;

fn animate_twist_modifier(
    mut query: Query<&mut SdModStack, With<AnimateTwitModifier>>,
    time: Res<Time>,
) {
    for mut stack in query.iter_mut() {
        for modifier in stack.modifiers.iter_mut().rev() {
            if let SdMod::Twist { k, .. } = modifier {
                let eas_func = EaseFunction::SmootherStep;
                *k = eas_func.sample_unchecked(time.elapsed_secs().sin().abs())
                    * time.elapsed_secs_wrapped().sin().signum()
                    / 50.0;
                return;
            }
        }
    }
}
