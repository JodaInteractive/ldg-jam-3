use bevy::{prelude::*, text::FontSmoothing};
use leafwing_input_manager::prelude::*;
use rand::random_range;

use crate::{
    PIXEL_PERFECT_LAYERS, ScaleFactor,
    audio::{PlaySfxEvent, PlaySoundtrackEvent, SoundEffect, Soundtrack},
    input::Action,
    screens::Screen,
    sundry::BLACK,
};

#[derive(Resource)]
pub struct GameStats {
    pub asteroids_destroyed: u32,
    pub distance_traveled: u32,
    pub shots_fired: u32,
    pub shots_hit: u32,
    // pub time_played: f32,
    // pub successful_trip: bool,
}

#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub enum GameState {
    #[default]
    Play,
    GameOver,
}

#[derive(Component)]
struct Thruster;

#[derive(Resource, PartialEq)]
struct HasEntered(bool);

pub(super) fn plugin(app: &mut App) {
    app.init_state::<GameState>();
    app.insert_resource(GameStats {
        asteroids_destroyed: 0,
        distance_traveled: 0,
        shots_fired: 0,
        shots_hit: 0,
        // time_played: 0.0,
        // successful_trip: false,
    });
    app.insert_resource::<HasEntered>(HasEntered(false));

    app.add_systems(
        OnEnter(Screen::Game),
        |mut has_entered: ResMut<HasEntered>| {
            has_entered.0 = false;
        },
    );

    app.add_systems(OnEnter(Screen::Game), trigger_music);
    app.add_systems(OnEnter(GameState::Play), spawn_game_screen);
    app.add_systems(
        FixedUpdate,
        enter_player.run_if(
            in_state(Screen::Game)
                .and(in_state(GameState::Play).and(resource_equals(HasEntered(false)))),
        ),
    );
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
            update_explosions,
        )
            .in_set(GameSystems::Play)
            .run_if(in_state(Screen::Game)),
    );

    app.add_systems(
        Update,
        (pause)
            .in_set(GameSystems::Play)
            .run_if(in_state(Screen::Game).and(in_state(GameState::Play))),
    );

    app.add_systems(
        Update,
        (game_over_input)
            .in_set(GameSystems::GameOver)
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

fn spawn_game_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut time: ResMut<Time<Virtual>>,
) {
    let default_speed = asset_server.load("sprites/ships/ship3-default.png");
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
        children![(
            Name::new("Player Ship Thruster"),
            Thruster,
            Sprite {
                image: default_speed.clone(),
                custom_size: Some(Vec2::splat(32.0)),
                ..default()
            }
        )],
    ));

    time.unpause();
}

fn enter_player(
    mut player: Query<(&mut Transform, &ShipSpeedSprites), With<Player>>,
    mut thruster: Query<&mut Sprite, With<Thruster>>,
    mut has_entered: ResMut<HasEntered>,
    mut game_stats: ResMut<GameStats>,
) {
    game_stats.asteroids_destroyed = 0;
    game_stats.distance_traveled = 0;
    game_stats.shots_fired = 0;
    game_stats.shots_hit = 0;
    let player = player.single_mut();
    if player.is_err() {
        return;
    }
    let (mut player_transform, ship_speed_sprites) = player.unwrap();
    player_transform.translation.y += 3.0;
    if player_transform.translation.y > -200.0 {
        player_transform.translation.y = -200.0;
        has_entered.0 = true;
    }
    let thruster = thruster.single_mut();
    if thruster.is_err() {
        return;
    }
    let mut thruster_sprite = thruster.unwrap();
    thruster_sprite.image = ship_speed_sprites.fast.clone();
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
    player: Query<&Transform, With<Player>>,
    asteroids: Query<&Transform, With<Asteroid>>,
    mut time: ResMut<Time<Virtual>>,
    mut state: ResMut<NextState<GameState>>,
) {
    let player = player.single();
    if player.is_err() {
        return;
    }
    let player_transform = player.unwrap();

    let hitbox_size = 8.0;

    for asteroid_transform in asteroids.iter() {
        let distance = player_transform
            .translation
            .distance(asteroid_transform.translation);
        if distance < (hitbox_size / 2.0) + (30.0 / 2.0) {
            time.pause();
            state.set(GameState::GameOver);
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

fn spawn_game_over(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    scale_factor: Res<ScaleFactor>,
    game_stats: Res<GameStats>,
) {
    let scale = scale_factor.0;
    let font_handle: Handle<Font> = asset_server.load("font.ttf");

    commands.spawn((
        Name::new("Game Over Screen"),
        Transform {
            translation: Vec3::new(0.0, 0.0, 100.0),
            ..default()
        },
        Node {
            width: Val::Percent(70.0),
            height: Val::Percent(70.0),
            position_type: PositionType::Absolute,
            justify_content: JustifyContent::Center,
            margin: UiRect::all(Val::Auto),
            ..default()
        },
        DespawnOnExit(Screen::Game),
        DespawnOnExit(GameState::GameOver),
        PIXEL_PERFECT_LAYERS,
        children![
            (
                Name::new("Game Over Text"),
                Text::new("GAME OVER"),
                TextFont {
                    font_size: 16.0 * scale,
                    font: font_handle.clone(),
                    font_smoothing: FontSmoothing::None,

                    ..default()
                },
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(8.0 * scale),
                    margin: UiRect {
                        left: Val::Auto,
                        right: Val::Auto,
                        ..default()
                    },
                    ..default()
                }
            ),
            (
                Name::new("Stats container"),
                Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                children![
                    stats_row(
                        "DISTANCE TRAVELED",
                        &game_stats.distance_traveled.to_string(),
                        scale,
                        font_handle.clone(),
                    ),
                    stats_row(
                        "SHOTS FIRED",
                        &game_stats.shots_fired.to_string(),
                        scale,
                        font_handle.clone(),
                    ),
                    stats_row(
                        "SHOTS HIT",
                        &game_stats.shots_hit.to_string(),
                        scale,
                        font_handle.clone()
                    ),
                    stats_row(
                        "ASTEROIDS DESTROYED",
                        &game_stats.asteroids_destroyed.to_string(),
                        scale,
                        font_handle.clone(),
                    ),
                ]
            ),
            (
                Name::new("Replay Button"),
                Button,
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(32.0 * scale),
                    ..default()
                },
                children![(
                    Name::new("Replay Button Text"),
                    Text::new("REPLAY"),
                    Node::default(),
                    TextFont {
                        font_size: 16.0 * scale,
                        font: font_handle.clone(),
                        font_smoothing: FontSmoothing::None,
                        ..default()
                    }
                )],
            ),
            (
                Name::new("Back to Menu Button"),
                Button,
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(8.0 * scale),
                    ..default()
                },
                children![(
                    Name::new("Back to Menu Text"),
                    Text::new("BACK TO MENU"),
                    Node::default(),
                    TextFont {
                        font_size: 16.0 * scale,
                        font: font_handle.clone(),
                        font_smoothing: FontSmoothing::None,

                        ..default()
                    }
                )]
            )
        ],
    ));
}

fn game_over_input(
    input_query: Query<&ActionState<Action>>,
    current_game_state: Res<State<GameState>>,
    mut state: ResMut<NextState<GameState>>,
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<Button>)>,
    mut has_entered: ResMut<HasEntered>,
    mut player: Query<&mut Transform, With<Player>>,
) {
    let action_state = input_query.single().unwrap();
    if action_state.just_pressed(&Action::Select)
        && current_game_state.get() == &GameState::GameOver
    {
        has_entered.0 = false;
        let mut player_transform = player.single_mut().unwrap();
        player_transform.translation = Vec3::new(0.0, -280.0, 0.0);
        state.set(GameState::Play);
    }

    for interaction in interaction_query.iter() {
        match *interaction {
            Interaction::Pressed => {
                has_entered.0 = false;
                let mut player_transform = player.single_mut().unwrap();
                player_transform.translation = Vec3::new(0.0, -280.0, 0.0);
                state.set(GameState::Play);
            }
            Interaction::Hovered => {}
            Interaction::None => {}
        }
    }
}

fn trigger_music(mut commands: Commands) {
    commands.trigger(PlaySoundtrackEvent {
        soundtrack: Soundtrack::BattleTheme,
    });
}

fn stats_row(stat: &str, value: &str, scale: f32, font_handle: Handle<Font>) -> impl Bundle {
    (
        Name::new(format!("{stat} Stat")),
        BackgroundColor(BLACK),
        Node {
            display: Display::Flex,
            justify_content: JustifyContent::SpaceBetween,
            top: Val::Px(32.0 * scale),
            width: Val::Percent(100.0),
            padding: UiRect {
                top: Val::Px(8.0 * scale),
                ..default()
            },
            margin: UiRect {
                left: Val::Px(32.0 * scale),
                right: Val::Px(32.0 * scale),
                ..default()
            },
            ..default()
        },
        children![
            (
                Name::new(format!("{stat} Label")),
                Text::new(stat),
                TextFont {
                    font_size: 16.0 * scale,
                    font: font_handle.clone(),
                    font_smoothing: FontSmoothing::None,
                    ..default()
                },
                Node {
                    left: Val::Px(0.0),
                    ..default()
                },
            ),
            (
                Name::new(format!("{stat} Value")),
                Text::new(value),
                TextFont {
                    font_size: 16.0 * scale,
                    font: font_handle.clone(),
                    font_smoothing: FontSmoothing::None,
                    ..default()
                },
                Node {
                    right: Val::Px(0.0),
                    ..default()
                }
            )
        ],
    )
}
