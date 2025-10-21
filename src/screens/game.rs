use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use rand::random_range;

use crate::{
    PIXEL_PERFECT_LAYERS,
    audio::{PlaySfxEvent, PlaySoundtrackEvent, SoundEffect, Soundtrack},
    input::Action,
    screens::{Screen, pause::PauseState},
};

#[derive(Resource)]
pub struct GameStats {
    pub asteroids_destroyed: u32,
    pub distance_traveled: u32,
    pub shots_fired: u32,
    pub shots_hit: u32,
    pub time_played: f32,
    // pub successful_trip: bool,
}

#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub enum GameState {
    #[default]
    Play,
    GameOver,
    EndGame,
}

#[derive(SystemSet, Debug, Clone, Hash, Eq, PartialEq)]
pub enum GameSystems {
    Play,
    Environment,
    GameOver,
    EndGame,
}

#[derive(Resource, PartialEq)]
pub struct HasEntered(pub bool);

pub(super) fn plugin(app: &mut App) {
    app.init_state::<GameState>();
    app.insert_resource(GameStats {
        asteroids_destroyed: 0,
        distance_traveled: 0,
        shots_fired: 0,
        shots_hit: 0,
        time_played: 0.0,
        // successful_trip: false,
    });
    app.insert_resource(HasEntered(false));
    app.insert_resource(EndGame(false));
    app.add_observer(restart_game);
    app.add_observer(back_to_menu);

    app.add_systems(OnEnter(Screen::Game), trigger_music);

    app.add_systems(Startup, (spawn_player, spawn_planet));
    app.configure_sets(Startup, GameSystems::Play);
    app.configure_sets(Startup, GameSystems::Environment);

    app.add_systems(
        Update,
        stopwatch.run_if(in_state(GameState::Play).and(in_state(PauseState::NotPaused))),
    );

    app.add_systems(
        FixedUpdate,
        run_endgame.run_if(
            in_state(GameState::Play)
                .and(in_state(PauseState::NotPaused))
                .and(resource_equals(EndGame(true))),
        ),
    );

    app.add_systems(
        FixedUpdate,
        enter_player.run_if(
            in_state(Screen::Game)
                .and(in_state(GameState::Play).and(resource_equals(HasEntered(false)))),
        ),
    );

    app.add_systems(
        FixedUpdate,
        (
            player_input,
            update_projectiles,
            projectile_asteroid_collision,
            update_explosions,
            shield_regen,
        )
            .in_set(GameSystems::Play)
            .run_if(in_state(Screen::Game)),
    );

    app.add_systems(
        FixedUpdate,
        (
            spawn_asteroids,
            update_asteroids,
            collide_with_asteroid_check,
        )
            .in_set(GameSystems::Environment)
            .run_if(in_state(Screen::Game)),
    );

    app.insert_resource(AsteroidSpawnTimer(Timer::from_seconds(
        1.0,
        TimerMode::Repeating,
    )));
}

#[derive(Component)]
struct Thruster;

#[derive(Component)]
struct Shield;

fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    let default_speed = asset_server.load("sprites/ships/ship3-default.png");
    commands.spawn((
        Name::new("Player Ship"),
        Player {
            can_shoot: true,
            timer: Timer::from_seconds(0.15, TimerMode::Once),
            shield: 2,
            shield_regen: Timer::from_seconds(5.0, TimerMode::Once),
            shield_hit_cooldown: Timer::from_seconds(1.0, TimerMode::Once),
        },
        Sprite {
            image: asset_server.load("sprites/ships/ship3.png"),
            custom_size: Some(Vec2::splat(32.0)),
            ..default()
        },
        ShipSpeedSprites {
            slow: asset_server.load("sprites/ships/ship3-slow.png"),
            default: default_speed.clone(),
            fast: asset_server.load("sprites/ships/ship3-fast.png"),
        },
        Transform {
            translation: Vec3::new(0.0, -280.0, 0.0),
            ..default()
        },
        PIXEL_PERFECT_LAYERS,
        children![
            (
                Name::new("Player Ship Thruster"),
                Thruster,
                Sprite {
                    image: default_speed.clone(),
                    custom_size: Some(Vec2::splat(32.0)),
                    ..default()
                }
            ),
            (
                Name::new("Player Shield"),
                Shield,
                Sprite {
                    image: asset_server.load("sprites/shield.png"),
                    ..default()
                }
            )
        ],
    ));
}

fn enter_player(
    mut player: Query<(&mut Transform, &ShipSpeedSprites), With<Player>>,
    mut thruster: Query<&mut Sprite, With<Thruster>>,
    mut has_entered: ResMut<HasEntered>,
    mut game_stats: ResMut<GameStats>,
    mut end_game: ResMut<EndGame>,
) {
    game_stats.asteroids_destroyed = 0;
    game_stats.distance_traveled = 0;
    game_stats.shots_fired = 0;
    game_stats.shots_hit = 0;
    game_stats.time_played = 0.0;
    println!("resetting end game");
    end_game.0 = false;
    let player = player.single_mut();
    if player.is_err() {
        return;
    }
    let (mut player_transform, ship_speed_sprites) = player.unwrap();
    player_transform.translation.y += 3.0;
    if player_transform.translation.y > -200.0 {
        player_transform.translation.y = -200.0;
        has_entered.0 = true;
        game_stats.time_played = 0.0;
    }
    let thruster = thruster.single_mut();
    if thruster.is_err() {
        return;
    }
    let mut thruster_sprite = thruster.unwrap();
    thruster_sprite.image = ship_speed_sprites.fast.clone();
}

#[derive(Component)]
pub struct Player {
    pub can_shoot: bool,
    pub timer: Timer,
    pub shield: u8,
    pub shield_regen: Timer,
    pub shield_hit_cooldown: Timer,
}

#[derive(Component)]
struct ShipSpeedSprites {
    slow: Handle<Image>,
    default: Handle<Image>,
    fast: Handle<Image>,
}

#[derive(Component)]
struct Projectile;

#[derive(Component)]
struct Asteroid {
    pub speed: f32,
    pub rotation_speed: f32,
    pub health: u8,
}

#[derive(Resource)]
struct AsteroidSpawnTimer(Timer);

#[derive(Resource, PartialEq)]
struct EndGame(bool);

fn stopwatch(
    mut commands: Commands,
    time: Res<Time<Virtual>>,
    mut game_stats: ResMut<GameStats>,
    mut end_game: ResMut<EndGame>,
) {
    println!("time played: {:?}", game_stats.time_played);
    if game_stats.time_played > 300.0 {
        game_stats.time_played = 300.0;
        end_game.0 = true;
        commands.trigger(PlaySoundtrackEvent {
            soundtrack: Soundtrack::End,
        });
    }
    game_stats.time_played += time.delta_secs();
}

#[derive(Component)]
struct Planet;

fn spawn_planet(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Planet,
        Sprite {
            image: asset_server.load("sprites/planet-smol.png"),
            ..default()
        },
        Transform {
            translation: Vec3 {
                x: 0.0,
                y: 400.0,
                z: -11.0,
            },
            ..default()
        },
    ));
}

fn run_endgame(
    mut planet: Single<&mut Transform, With<Planet>>,
    mut time: ResMut<Time<Virtual>>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    planet.translation.y -= 0.5;
    if planet.translation.y < -100.0 {
        planet.translation.y = -100.0;
        time.pause();
        game_state.set(GameState::EndGame);
    }
}

fn player_input(
    mut commands: Commands,
    input_query: Query<&ActionState<Action>>,
    mut player_query: Query<(&mut Transform, &mut Player, &ShipSpeedSprites)>,
    mut thruster: Query<&mut Sprite, With<Thruster>>,
    time: Res<Time<Virtual>>,
    asset_server: Res<AssetServer>,
    mut game_stats: ResMut<GameStats>,
) {
    let action_state = input_query.single().unwrap();
    let player = player_query.single_mut();
    if player.is_err() {
        return;
    }
    let (mut player_transform, mut player, ship_speed_sprites) = player.unwrap();

    player.timer.tick(time.delta());
    if player.timer.is_finished() {
        player.can_shoot = true;
    }

    let player_speed = 3.0;

    let mut intent = Vec2::ZERO;

    if action_state.pressed(&Action::Up) {
        intent += Vec2::Y;
    }

    if action_state.pressed(&Action::Down) {
        intent -= Vec2::Y;
    }

    if action_state.pressed(&Action::Left) {
        intent -= Vec2::X;
    }

    if action_state.pressed(&Action::Right) {
        intent += Vec2::X;
    }

    if intent != Vec2::ZERO {
        player_transform.translation += intent.normalize().extend(0.0) * player_speed;
        player_transform.translation.x = player_transform.translation.x.clamp(-400.0, 400.0);
        player_transform.translation.y = player_transform.translation.y.clamp(-220.0, 220.0);
    }

    let thruster = thruster.single_mut();
    if let Ok(mut thruster_sprite) = thruster {
        if intent.y > 0.0 {
            thruster_sprite.image = ship_speed_sprites.fast.clone();
        } else if intent.y < 0.0 {
            thruster_sprite.image = ship_speed_sprites.slow.clone();
        } else {
            thruster_sprite.image = ship_speed_sprites.default.clone();
        }
    }

    if action_state.pressed(&Action::Shoot) && player.can_shoot {
        player.can_shoot = false;
        player.timer.reset();
        game_stats.shots_fired += 1;
        commands.trigger(PlaySfxEvent {
            sfx: SoundEffect::Shoot,
        });
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
    end_game: Res<EndGame>,
) {
    if end_game.0 {
        return;
    }
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
    let health = if size.x > 32.0 { 5 } else { 2 };
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
            health,
            speed: random_range(1.0..2.0),
            rotation_speed: random_range(-0.02..0.02),
        },
        DespawnOnExit(Screen::Game),
    ));
}

fn collide_with_asteroid_check(
    mut commands: Commands,
    mut player: Single<(&Transform, &mut Player)>,
    asteroids: Query<&Transform, With<Asteroid>>,
    mut time: ResMut<Time<Virtual>>,
    mut state: ResMut<NextState<GameState>>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    mut shield: Single<&mut Sprite, With<Shield>>,
) {
    let player_transform = player.0;
    let player = &mut player.1;
    let hitbox_size = 8.0;
    for asteroid_transform in asteroids.iter() {
        let distance = player_transform
            .translation
            .distance(asteroid_transform.translation);
        if distance < (hitbox_size / 2.0) + (30.0 / 2.0) && player.shield_hit_cooldown.is_finished()
        {
            if player.shield == 0 {
                time.pause();
                state.set(GameState::GameOver);
            } else {
                player.shield -= 1;
                player.shield_regen.reset();
                player.shield_hit_cooldown.reset();
                if player.shield == 0 {
                    shield.image = asset_server.load("sprites/shield-empty.png");
                } else if player.shield == 1 {
                    shield.image = asset_server.load("sprites/shield-low.png");
                }
            }

            commands.trigger(PlaySfxEvent {
                sfx: SoundEffect::Explosion,
            });
            let texture_atlas_layout =
                TextureAtlasLayout::from_grid(UVec2::splat(32), 8, 1, None, None);
            let texture_atlas_handle = texture_atlases.add(texture_atlas_layout);
            commands.trigger(PlaySfxEvent {
                sfx: SoundEffect::Explosion,
            });
            commands.spawn((
                Name::new("Explosion"),
                Sprite::from_atlas_image(
                    asset_server.load("sprites/explosion.png"),
                    TextureAtlas::from(texture_atlas_handle),
                ),
                Transform {
                    translation: player_transform.translation,
                    ..default()
                },
                DespawnOnExit(Screen::Game),
                Explosion,
                PIXEL_PERFECT_LAYERS,
            ));
        }
    }
}

fn shield_regen(
    time: Res<Time<Virtual>>,
    mut player: Single<&mut Player>,
    mut shield: Single<&mut Sprite, With<Shield>>,
    asset_server: Res<AssetServer>,
) {
    player.shield_regen.tick(time.delta());
    player.shield_hit_cooldown.tick(time.delta());
    if player.shield_regen.just_finished() {
        player.shield_regen.reset();
        player.shield += 1;
        if player.shield > 2 {
            player.shield = 2;
        }
        match player.shield {
            0 => shield.image = asset_server.load("sprites/shield-empty.png"),
            1 => shield.image = asset_server.load("sprites/shield-low.png"),
            2 => shield.image = asset_server.load("sprites/shield.png"),
            _ => {}
        }
    }
}

#[derive(Component)]
struct Explosion;

fn projectile_asteroid_collision(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    projectiles: Query<(Entity, &Transform), With<Projectile>>,
    mut asteroids: Query<(Entity, &Transform, &mut Asteroid)>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    mut game_stats: ResMut<GameStats>,
) {
    for (projectile_entity, projectile_transform) in projectiles.iter() {
        for (asteroid_entity, asteroid_transform, mut asteroid) in asteroids.iter_mut() {
            let distance = projectile_transform
                .translation
                .distance(asteroid_transform.translation);
            if distance < (16.0 / 2.0) + (30.0 / 2.0) {
                game_stats.shots_hit += 1;
                asteroid.health -= 1;
                if asteroid.health == 0 {
                    game_stats.asteroids_destroyed += 1;
                    commands.entity(asteroid_entity).despawn();
                }
                commands.entity(projectile_entity).despawn();
                let texture_atlas_layout =
                    TextureAtlasLayout::from_grid(UVec2::splat(32), 8, 1, None, None);
                let texture_atlas_handle = texture_atlases.add(texture_atlas_layout);
                commands.trigger(PlaySfxEvent {
                    sfx: SoundEffect::Explosion,
                });
                commands.spawn((
                    Name::new("Explosion"),
                    Sprite::from_atlas_image(
                        asset_server.load("sprites/explosion.png"),
                        TextureAtlas::from(texture_atlas_handle),
                    ),
                    Transform {
                        translation: projectile_transform.translation,
                        ..default()
                    },
                    DespawnOnExit(Screen::Game),
                    Explosion,
                    PIXEL_PERFECT_LAYERS,
                ));
            }
        }
    }
}

fn update_explosions(
    mut commands: Commands,
    mut explosions: Query<(Entity, &mut Transform, &mut Sprite), With<Explosion>>,
) {
    for (entity, mut transform, mut sprite) in explosions.iter_mut() {
        transform.translation -= Vec3::Y * 2.0;
        let atlas = sprite.texture_atlas.as_mut().unwrap();
        atlas.index += 1;
        if atlas.index >= 8 {
            commands.entity(entity).despawn();
        }
        sprite.texture_atlas = Some(atlas.clone());
    }
}

fn trigger_music(mut commands: Commands) {
    commands.trigger(PlaySoundtrackEvent {
        soundtrack: Soundtrack::Battle,
    });
}

#[derive(Event)]
pub struct RestartGame;

fn restart_game(
    _event: On<RestartGame>,
    mut commands: Commands,
    mut time: ResMut<Time<Virtual>>,
    mut pause_state: ResMut<NextState<PauseState>>,
    mut game_state: ResMut<NextState<GameState>>,
    mut has_entered: ResMut<HasEntered>,
    mut player: Query<&mut Transform, With<Player>>,
    asteroids: Query<Entity, With<Asteroid>>,
    projectiles: Query<Entity, With<Projectile>>,
    mut end_game: ResMut<EndGame>,
    mut planet: Query<&mut Transform, (With<Planet>, Without<Player>)>,
) {
    for asteroid in asteroids.iter() {
        commands.entity(asteroid).despawn();
    }

    for projectile in projectiles.iter() {
        commands.entity(projectile).despawn();
    }

    end_game.0 = false;
    pause_state.set(PauseState::NotPaused);
    let mut player_transform = player.single_mut().unwrap();
    player_transform.translation = Vec3::new(0.0, -280.0, 0.0);

    let mut planet_transform = planet.single_mut().unwrap();
    planet_transform.translation = Vec3::new(0.0, 400.0, -11.0);

    has_entered.0 = false;
    game_state.set(GameState::Play);
    time.unpause();
}

#[derive(Event)]
pub struct BackToMenuEvent;

fn back_to_menu(
    _event: On<BackToMenuEvent>,
    mut commands: Commands,
    mut screen_state: ResMut<NextState<Screen>>,
) {
    commands.trigger(RestartGame);
    screen_state.set(Screen::Title);
}
