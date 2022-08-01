#![deny(clippy::all)]
#![warn(clippy::pedantic, clippy::cargo)]
#![allow(
    clippy::module_name_repetitions,
    clippy::cargo_common_metadata,
    clippy::type_complexity,
    clippy::too_many_arguments,
    clippy::needless_pass_by_value,
    clippy::multiple_crate_versions,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::too_many_lines,
    clippy::similar_names,
    clippy::must_use_candidate,
    clippy::enum_glob_use
)]

use std::f32::consts::PI;

use bevy::{
    prelude::*,
    window::{close_on_esc, PresentMode},
};

pub const CLEAR: Color = Color::BLACK;
pub const HEIGHT: f32 = 600.0;
pub const RESOLUTION: f32 = 16.0 / 9.0;

fn main() {
    App::new()
        .insert_resource(ClearColor(CLEAR))
        .insert_resource(WindowDescriptor {
            width: HEIGHT * RESOLUTION,
            height: HEIGHT,
            title: "Bevy Template".to_string(),
            present_mode: PresentMode::Fifo,
            resizable: false,
            position: WindowPosition::Centered(MonitorSelection::Number(0)),
            ..Default::default()
        })
        // External plugins
        .add_plugins(DefaultPlugins)
        .add_system(close_on_esc)
        // Internal plugins
        .add_startup_system(spawn_camera)
        .add_startup_system(spawn_stage)
        .add_startup_system(spawn_character)
        .run();
}

fn spawn_camera(mut commands: Commands) {
    let mut camera = Camera3dBundle::default();

    camera.transform.translation = Vec3::new(10.0, 0.0, 10.0);
    camera.transform.look_at(Vec3::ZERO, Vec3::NEG_X);
    camera.transform.translation.x -= 3.0;

    commands.spawn_bundle(camera);
}

fn spawn_stage(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut mat: StandardMaterial = Color::PURPLE.into();
    mat.unlit = true;
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(
            shape::Quad {
                size: Vec2 { x: 5.0, y: 30.0 },
                flip: false,
            }
            .into(),
        ),
        material: materials.add(mat),
        ..default()
    });
}

fn spawn_character(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut mat: StandardMaterial = Color::GREEN.into();
    mat.unlit = true;
    let mut transform = Transform::default().looking_at(Vec3::new(7.0, 0.0, 10.0), Vec3::NEG_X);
    transform.translation = Vec3::new(0.0, 0.0, 1.0);
    transform.rotate_axis(Vec3::Y, PI * 0.56);
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(shape::Plane { size: 1.0 }.into()),
        transform,
        material: materials.add(mat),
        ..default()
    });
}
