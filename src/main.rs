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
    render::{camera::Projection, texture::ImageSettings},
    window::{close_on_esc, PresentMode},
};
use bevy_asset_loader::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_sprite3d::{AtlasSprite3d, Sprite3dParams, Sprite3dPlugin};
use leafwing_input_manager::prelude::*;

pub const CLEAR: Color = Color::BLACK;
pub const HEIGHT: f32 = 600.0;
pub const RESOLUTION: f32 = 16.0 / 9.0;

fn main() {
    App::new()
        .add_loading_state(
            LoadingState::new(GameState::Loading)
                .continue_to_state(GameState::Ready)
                .with_collection::<ImageAssets>(),
        )
        .insert_resource(ImageSettings::default_nearest())
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
        .add_state(GameState::Loading)
        // External plugins
        .add_plugins(DefaultPlugins)
        .add_plugin(Sprite3dPlugin)
        .insert_resource(RapierConfiguration {
            gravity: Vect::Z * -9.81,
            ..default()
        })
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_system(close_on_esc)
        .add_plugin(InputManagerPlugin::<Action>::default())
        // Internal plugins
        .add_system_set(
            SystemSet::on_enter(GameState::Ready)
                .with_system(spawn_camera)
                .with_system(spawn_stage)
                .with_system(spawn_character),
        )
        .add_system_set(
            SystemSet::on_update(GameState::Ready)
                .with_system(player_control)
                .with_system(character_state),
        )
        .run();
}

fn spawn_camera(mut commands: Commands) {
    let mut camera = Camera3dBundle {
        projection: Projection::Perspective(PerspectiveProjection {
            fov: PI / 6.0,
            ..default()
        }),
        ..default()
    };

    camera.transform.translation = Vec3::new(10.0, 0.0, 5.0);
    camera.transform.look_at(Vec3::ZERO, Vec3::NEG_X);

    commands.spawn_bundle(camera);
}

#[derive(Component)]
struct Ground;

fn spawn_stage(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut mat: StandardMaterial = Color::PURPLE.into();
    mat.unlit = true;
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(
                shape::Quad {
                    size: Vec2 { x: 5.0, y: 30.0 },
                    flip: false,
                }
                .into(),
            ),
            material: materials.add(mat),
            ..default()
        })
        .insert_bundle((Collider::cuboid(2.5, 15.0, 0.01), RigidBody::Fixed, Ground));
}

fn spawn_character(
    mut commands: Commands,
    images: Res<ImageAssets>,
    mut sprite_params: Sprite3dParams,
) {
    let mut transform = Transform::from_translation(Vec3::new(0.0, 0.0, 0.25))
        .with_rotation(Quat::from_axis_angle(Vec3::Z, PI * 0.5));
    transform.rotate(Quat::from_axis_angle(Vec3::Y, PI * 0.5));
    commands
        .spawn_bundle(
            AtlasSprite3d {
                atlas: images.character_sprite.clone(),
                partial_alpha: true,
                transform,
                unlit: true,
                pivot: Some(Vec2::new(0.7, 0.5)),
                ..default()
            }
            .bundle(&mut sprite_params),
        )
        .insert_bundle(InputManagerBundle::<Action> {
            input_map: InputMap::new([
                (KeyCode::A, Action::MoveLeft),
                (KeyCode::E, Action::MoveRight),
                (KeyCode::O, Action::MoveTowards),
                (KeyCode::Comma, Action::MoveAway),
                (KeyCode::Space, Action::Jump),
            ]),
            ..default()
        })
        .insert_bundle((
            Collider::cuboid(0.25, 0.25, 0.25),
            RigidBody::Dynamic,
            LockedAxes::ROTATION_LOCKED,
            Velocity::default(),
            ExternalImpulse::default(),
            ActiveEvents::COLLISION_EVENTS,
            Player,
            CharacterState::Grounded,
        ));
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum GameState {
    Loading,
    Ready,
}

#[derive(AssetCollection)]
struct ImageAssets {
    #[asset(texture_atlas(tile_size_x = 64., tile_size_y = 64., columns = 14, rows = 7,))]
    #[asset(path = "player/PlayerSharky(64x64).png")]
    character_sprite: Handle<TextureAtlas>,
}

#[derive(Component)]
struct Player;

#[allow(clippy::enum_variant_names)]
#[derive(Actionlike, Clone, Copy)]
enum Action {
    MoveLeft,
    MoveRight,
    MoveAway,
    MoveTowards,
    Jump,
}

fn player_control(
    mut player: Query<
        (
            &mut Velocity,
            &mut ExternalImpulse,
            &ActionState<Action>,
            &mut CharacterState,
        ),
        With<Player>,
    >,
) {
    let (mut velocity, mut impulse, action_state, mut character_state) = player.single_mut();
    let mut movement = Vec2::default();
    for action in action_state.get_pressed() {
        match action {
            Action::MoveLeft => movement.y = -1.0,
            Action::MoveRight => movement.y = 1.0,
            Action::MoveAway => movement.x = -1.0,
            Action::MoveTowards => movement.x = 1.0,
            Action::Jump => {
                if matches!(*character_state, CharacterState::Grounded) {
                    impulse.impulse = Vec3::new(0.0, 0.0, 0.7);
                    *character_state = CharacterState::InAir;
                }
            }
        }
    }
    velocity.linvel = (movement.normalize_or_zero() * 10.0).extend(velocity.linvel.z);
}

#[derive(Component, Clone, Copy)]
enum CharacterState {
    Grounded,
    InAir,
}

fn character_state(
    mut events: EventReader<CollisionEvent>,
    mut characters: Query<&mut CharacterState>,
    ground: Query<(), With<Ground>>,
) {
    for event in events.iter() {
        match event {
            CollisionEvent::Started(e1, e2, _) => {
                let mut character = if let Ok(character) = characters.get_mut(*e1) {
                    if ground.contains(*e2) {
                        character
                    } else {
                        continue;
                    }
                } else if let Ok(character) = characters.get_mut(*e2) {
                    if ground.contains(*e1) {
                        character
                    } else {
                        continue;
                    }
                } else {
                    continue;
                };
                *character = CharacterState::Grounded;
            }
            CollisionEvent::Stopped(e1, e2, _) => {
                let mut character = if let Ok(character) = characters.get_mut(*e1) {
                    if ground.contains(*e2) {
                        character
                    } else {
                        continue;
                    }
                } else if let Ok(character) = characters.get_mut(*e2) {
                    if ground.contains(*e1) {
                        character
                    } else {
                        continue;
                    }
                } else {
                    continue;
                };
                *character = CharacterState::InAir;
            }
        }
    }
}
