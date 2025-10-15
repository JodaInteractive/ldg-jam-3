use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use rand::random_range;

use crate::{PIXEL_PERFECT_LAYERS, input::Action, screens::Screen};

#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub enum GameState {
    #[default]
    Play,
    GameOver,
}

pub(super) fn plugin(app: &mut App) {
    app.init_state::<GameState>();
    app.add_systems(OnEnter(GameState::Play), spawn_game_screen);
    app.add_systems(OnEnter(GameState::GameOver), spawn_game_over);
    app.add_systems(OnExit(GameState::GameOver), despawn_game_screen);
    app.configure_sets(Startup, GameSystems::Play);
    app.configure_sets(Startup, GameSystems::Environment);
    app.configure_sets(Startup, GameSystems::GameOver);

    app.add_systems(
        FixedUpdate,
        (
            player_input,
            update_projectiles,
            projectile_asteroid_collision,
        )
            .in_set(GameSystems::Play),
    );

    app.add_systems(Update, (pause).in_set(GameSystems::Play));

    app.add_systems(Update, (game_over_input).in_set(GameSystems::GameOver));

    app.add_systems(
        FixedUpdate,
        (
            spawn_asteroids,
            update_asteroids,
            collide_with_asteroid_check,
        )
            .in_set(GameSystems::Environment),
    );

    app.insert_resource(AsteroidSpawnTimer(Timer::from_seconds(
        1.0,
        TimerMode::Repeating,
    )));
}

fn spawn_game_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut time: ResMut<Time<Virtual>>,
) {
    commands.spawn((
        Name::new("Player Ship"),
        Player {
            can_shoot: true,
            timer: Timer::from_seconds(0.15, TimerMode::Once),
        },
        DespawnOnExit(Screen::Game),
        Sprite {
            image: asset_server.load("sprites/ships/ship3.png"),
            custom_size: Some(Vec2::splat(32.0)),
            ..default()
        },
        Transform {
            translation: Vec3::new(0.0, -200.0, 0.0),
            ..default()
        },
        PIXEL_PERFECT_LAYERS,
    ));

    time.unpause();
}

fn despawn_game_screen(
    mut commands: Commands,
    player: Query<Entity, With<Player>>,
    asteraids: Query<Entity, With<Asteroid>>,
    projectiles: Query<Entity, With<Projectile>>,
) {
    let player = player.single().unwrap();
    commands.entity(player).despawn();

    for asteroid in asteraids.iter() {
        commands.entity(asteroid).despawn();
    }

    for projectile in projectiles.iter() {
        commands.entity(projectile).despawn();
    }
}

#[derive(SystemSet, Debug, Clone, Hash, Eq, PartialEq)]
enum GameSystems {
    Play,
    Environment,
    GameOver,
}

#[derive(Component)]
struct Player {
    pub can_shoot: bool,
    pub timer: Timer,
}

#[derive(Component)]
struct Projectile;

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
    mut player_query: Query<(&mut Transform, &mut Player)>,
    time: Res<Time<Virtual>>,
    asset_server: Res<AssetServer>,
) {
    let action_state = input_query.single().unwrap();
    let (mut player_transform, mut player) = player_query.single_mut().unwrap();

    player.timer.tick(time.delta());
    if player.timer.is_finished() {
        player.can_shoot = true;
    }

    let player_speed = 6.0;

    if action_state.pressed(&Action::Up) {
        player_transform.translation += Vec3::Y * player_speed;
    }

    if action_state.pressed(&Action::Down) {
        player_transform.translation -= Vec3::Y * player_speed;
    }

    if action_state.pressed(&Action::Left) {
        player_transform.translation -= Vec3::X * player_speed;
    }

    if action_state.pressed(&Action::Right) {
        player_transform.translation += Vec3::X * player_speed;
    }

    player_transform.translation.x = player_transform.translation.x.clamp(-400.0, 400.0);
    player_transform.translation.y = player_transform.translation.y.clamp(-220.0, 220.0);

    if action_state.pressed(&Action::Shoot) && player.can_shoot {
        commands.spawn((
            Name::new("Player Projectile"),
            PIXEL_PERFECT_LAYERS,
            Sprite {
                image: asset_server.load("sprites/projectiles/projectile1.png"),
                ..default()
            },
            Transform {
                translation: player_transform.translation + Vec3::Y * 20.0,
                ..default()
            },
            Projectile,
            DespawnOnExit(Screen::Game),
        ));
        player.can_shoot = false;
        player.timer.reset();
    }
}

fn pause(mut time: ResMut<Time<Virtual>>, input_query: Query<&ActionState<Action>>) {
    let action_state = input_query.single().unwrap();
    if action_state.just_pressed(&Action::Pause) {
        if time.is_paused() {
            time.unpause();
        } else {
            time.pause();
        }
    }
}

fn update_projectiles(mut projectiles: Query<&mut Transform, With<Projectile>>) {
    for mut projectile in projectiles.iter_mut() {
        projectile.translation += Vec3::Y * 15.0;
    }
}

fn spawn_asteroids(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut timer: ResMut<AsteroidSpawnTimer>,
    time: Res<Time<Virtual>>,
) {
    timer.0.tick(time.delta());
    if !timer.0.is_finished() {
        return;
    }

    let count = random_range(3..=9);

    let lanes = 10;
    let lane_width = 750.0 / lanes as f32;
    let mut used_lanes = Vec::new();

    let mut spawned_count = 0;
    if count > 6 {
        let big_spawn = random_range(0..4) == 0;
        if big_spawn {
            let lane = random_range(1..(lanes - 1));
            used_lanes.push(lane);
            let lane_variance = random_range(-lane_width / 4.0..lane_width / 4.0);
            let x = lane_width * lane as f32 + lane_width / 2.0 + lane_variance;
            spawn_asteroid(
                &mut commands,
                &asset_server,
                Vec3::new(x - 350.0, 360.0, -10.0),
                Vec2::splat(64.0),
            );
        }
        spawned_count += 2;
    }

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
            Vec3::new(x - 350.0, 360.0, -10.0),
            Vec2::splat(32.0),
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
            continue;
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
    let image_index = random_range(3..=4);
    let image = format!("sprites/asteroids/asteroid{image_index}.png");
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
            speed: random_range(1.0..2.0),
            rotation_speed: random_range(-0.02..0.02),
        },
        DespawnOnExit(Screen::Game),
    ));
}

fn collide_with_asteroid_check(
    player: Query<&Transform, With<Player>>,
    asteroids: Query<&Transform, With<Asteroid>>,
    mut time: ResMut<Time<Virtual>>,
    mut state: ResMut<NextState<GameState>>,
) {
    let player_transform = player.single().unwrap();
    let hitbox_size = 8.0;

    for asteroid_transform in asteroids.iter() {
        let distance = player_transform
            .translation
            .distance(asteroid_transform.translation);
        if distance < (hitbox_size / 2.0) + (30.0 / 2.0) {
            println!("Player hit by an asteroid!");
            time.pause();
            state.set(GameState::GameOver);
        }
    }
}

fn projectile_asteroid_collision(
    mut commands: Commands,
    projectiles: Query<(Entity, &Transform), With<Projectile>>,
    asteroids: Query<(Entity, &Transform), With<Asteroid>>,
) {
    for (projectile_entity, projectile_transform) in projectiles.iter() {
        for (asteroid_entity, asteroid_transform) in asteroids.iter() {
            let distance = projectile_transform
                .translation
                .distance(asteroid_transform.translation);
            if distance < (16.0 / 2.0) + (30.0 / 2.0) {
                commands.entity(projectile_entity).despawn();
                commands.entity(asteroid_entity).despawn();
            }
        }
    }
}

fn spawn_game_over(mut commands: Commands) {
    commands.spawn((
        Name::new("Game Over Screen"),
        Transform {
            translation: Vec3::new(0.0, 0.0, 100.0),
            ..default()
        },
        Node::default(),
        DespawnOnExit(Screen::Game),
        DespawnOnExit(GameState::GameOver),
        PIXEL_PERFECT_LAYERS,
        children![
            (
                Name::new("Game Over Text"),
                Text::new("Game Over"),
                Node::default()
            ),
            (
                Name::new("Replay Button"),
                Button,
                Node {
                    margin: UiRect::all(Val::Px(10.0)),
                    padding: UiRect::all(Val::Px(10.0)),
                    ..default()
                },
                BackgroundColor(Color::BLACK),
                children![(Name::new("Replay Button Text"), Text::new("Replay"))],
            )
        ],
    ));
}

fn game_over_input(
    input_query: Query<&ActionState<Action>>,
    mut state: ResMut<NextState<GameState>>,
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<Button>)>,
) {
    let action_state = input_query.single().unwrap();
    if action_state.just_pressed(&Action::Select) {
        state.set(GameState::Play);
    }

    for interaction in interaction_query.iter() {
        match *interaction {
            Interaction::Pressed => {
                state.set(GameState::Play);
            }
            Interaction::Hovered => {}
            Interaction::None => {}
        }
    }
}
