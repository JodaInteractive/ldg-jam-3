use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use rand::random_range;

use crate::{input::Action, screens::Screen};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Game), spawn_game_screen);
    app.configure_sets(Startup, GameSet::Player);
    app.configure_sets(Startup, GameSet::Environment);

    app.add_systems(
        FixedUpdate,
        (player_input, update_projectiles).in_set(GameSet::Player),
    );

    app.add_systems(
        FixedUpdate,
        (spawn_asteroids, update_asteroids).in_set(GameSet::Environment),
    );

    app.insert_resource(AsteroidSpawnTimer(Timer::from_seconds(
        1.0,
        TimerMode::Repeating,
    )));
}

fn spawn_game_screen(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Name::new("Player Ship"),
        Player,
        DespawnOnExit(Screen::Game),
        Sprite {
            image: asset_server.load("sprites/ships/ship1.png"),
            custom_size: Some(Vec2::splat(64.0)),
            ..default()
        },
        PlayerShotTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        children![(
            Name::new("Player Hitbox"),
            Sprite {
                image: asset_server.load("sprites/ships/ship1-hitbox.png"),
                custom_size: Some(Vec2::splat(64.0)),
                ..default()
            }
        )],
    ));
}

#[derive(SystemSet, Debug, Clone, Hash, Eq, PartialEq)]
enum GameSet {
    Player,
    Environment,
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Projectile;

#[derive(Component)]
struct PlayerShotTimer(Timer);

#[derive(Component)]
struct Asteroid {
    pub speed: f32,
    pub rotation_speed: f32,
}

#[derive(Resource)]
struct AsteroidSpawnTimer(Timer);

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

fn spawn_asteroids(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut timer: ResMut<AsteroidSpawnTimer>,
    time: Res<Time>,
) {
    timer.0.tick(time.delta());
    if !timer.0.is_finished() {
        return;
    }

    let count = random_range(1..=5);

    let lanes = 7;
    let lane_width = 1400.0 / lanes as f32;
    let mut used_lanes = Vec::new();

    if count == 5 {
        let big_spawn = random_range(0..4) == 0;
        if big_spawn {
            let lane = random_range(1..(lanes - 1));
            used_lanes.push(lane);
            used_lanes.push(lane - 1);
            used_lanes.push(lane + 1);
            let lane_variance = random_range(-lane_width / 4.0..lane_width / 4.0);
            let x = lane_width * lane as f32 + lane_width / 2.0 + lane_variance;
            spawn_asteroid(
                &mut commands,
                &asset_server,
                Vec3::new(x - 700.0, 720.0, -10.0),
                Vec2::splat(256.0),
            );
            return;
        }
    }

    let mut spawned_count = 0;
    while spawned_count < count {
        let lane = random_range(0..lanes);
        if used_lanes.contains(&lane) {
            continue;
        }
        let lane_variance = random_range(-lane_width / 4.0..lane_width / 4.0);
        let x = lane_width * lane as f32 + lane_width / 2.0 + lane_variance;
        spawn_asteroid(
            &mut commands,
            &asset_server,
            Vec3::new(x - 700.0, 720.0, -10.0),
            Vec2::splat(128.0),
        );
        spawned_count += 1;
        used_lanes.push(lane);
    }
}

fn update_asteroids(
    mut commands: Commands,
    mut asteroids: Query<(Entity, &mut Transform, &Asteroid)>,
) {
    for (entity, mut transform, asteroid) in asteroids.iter_mut() {
        if transform.translation.y < -800.0 {
            commands.entity(entity).despawn();
        }
        transform.translation -= Vec3::Y * asteroid.speed;
        transform.rotation *= Quat::from_rotation_z(asteroid.rotation_speed);
    }
}

fn spawn_asteroid(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    position: Vec3,
    size: Vec2,
) {
    let image_index = random_range(1..=2);
    let image = format!("sprites/asteroids/asteroid{}.png", image_index);
    commands.spawn((
        Name::new("Asteroid"),
        Sprite {
            image: asset_server.load(image),
            custom_size: Some(size),
            flip_x: rand::random_bool(0.5),
            flip_y: rand::random_bool(0.5),
            ..default()
        },
        Transform {
            translation: position,
            rotation: Quat::from_rotation_z(random_range(0.0..std::f32::consts::TAU)),
            ..default()
        },
        Asteroid {
            speed: random_range(1.0..4.0),
            rotation_speed: random_range(-0.02..0.02),
        },
        DespawnOnExit(Screen::Game),
    ));
}
