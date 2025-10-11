use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::{input::Action, screens::Screen};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Game), spawn_game_screen);
    app.configure_sets(Startup, GameSet::Player);

    app.add_systems(
        Update,
        (player_input, update_projectiles).in_set(GameSet::Player),
    );
}

fn spawn_game_screen(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Name::new("Player Ship"),
        Player,
        DespawnOnExit(Screen::Game),
        Sprite {
            image: asset_server.load("sprites/ships/ship1.png"),
            ..default()
        },
        PlayerShotTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        children![(
            Name::new("Player Hitbox"),
            Sprite {
                image: asset_server.load("sprites/ships/ship1-hitbox.png"),
                ..default()
            }
        )],
    ));
}

#[derive(SystemSet, Debug, Clone, Hash, Eq, PartialEq)]
enum GameSet {
    Player,
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Projectile;

#[derive(Component)]
struct PlayerShotTimer(Timer);

fn player_input(
    mut commands: Commands,
    input_query: Query<&ActionState<Action>>,
    mut player: Query<(&mut Transform, &mut PlayerShotTimer), With<Player>>,
    time: Res<Time>,
) {
    let action_state = input_query.single().unwrap();
    let (mut player_transform, mut shot_timer) = player.single_mut().unwrap();

    shot_timer.0.tick(time.delta());

    if action_state.pressed(&Action::Up) {
        player_transform.translation += Vec3::Y * 5.0;
    }

    if action_state.pressed(&Action::Down) {
        player_transform.translation -= Vec3::Y * 5.0;
    }

    if action_state.pressed(&Action::Left) {
        player_transform.translation -= Vec3::X * 5.0;
    }

    if action_state.pressed(&Action::Right) {
        player_transform.translation += Vec3::X * 5.0;
    }

    if action_state.pressed(&Action::Shoot) && shot_timer.0.is_finished() {
        commands.spawn((
            Name::new("Player Projectile"),
            Sprite {
                custom_size: Some(Vec2::splat(16.0)),
                ..default()
            },
            Transform {
                translation: player_transform.translation + Vec3::Y * 20.0,
                ..default()
            },
            Projectile,
            DespawnOnExit(Screen::Game),
        ));
    }
}

fn update_projectiles(mut projectiles: Query<&mut Transform, With<Projectile>>) {
    for mut projectile in projectiles.iter_mut() {
        projectile.translation += Vec3::Y * 10.0;
    }
}
