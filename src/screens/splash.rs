use bevy::prelude::*;
use rand::random_range;

use crate::{
    PIXEL_PERFECT_LAYERS,
    screens::{Screen, game::Star},
};

#[derive(Resource)]
struct StarPool {
    active_count: usize,
    inactive_stars: Vec<Entity>,
}

#[derive(Component)]
struct Title;

#[derive(Component)]
struct Developer;

#[derive(Component)]
struct FadeOut;

pub(super) fn plugin(app: &mut App) {
    app.insert_resource::<StarPool>(StarPool {
        active_count: 0,
        inactive_stars: Vec::new(),
    });
    app.add_systems(OnEnter(Screen::Splash), setup_splash);
    app.add_systems(
        FixedUpdate,
        (spawn_stars, update_stars).run_if(in_state(Screen::Splash).or(in_state(Screen::Title))),
    );
    app.add_systems(
        FixedUpdate,
        (splash_update, update_title, update_developer, fade_out).run_if(in_state(Screen::Splash)),
    );
}

fn setup_splash(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut star_pool: ResMut<StarPool>,
) {
    commands.spawn((
        Title,
        Sprite {
            image: asset_server.load("sprites/title.png"),
            color: Color::srgb_u8(235, 237, 233),
            custom_size: Some(Vec2::new(256.0, 64.0)),
            ..default()
        },
        DespawnOnExit(Screen::Title),
        Transform {
            translation: Vec3::new(0.0, 280.0, 0.0),
            ..default()
        },
        PIXEL_PERFECT_LAYERS,
    ));

    commands.spawn((
        Developer,
        Sprite {
            image: asset_server.load("sprites/jodainteractive.png"),
            color: Color::srgb_u8(235, 237, 233),
            custom_size: Some(Vec2::new(512.0, 64.0)),
            ..default()
        },
        DespawnOnExit(Screen::Splash),
        Transform {
            translation: Vec3::new(0.0, 280.0, 0.0),
            ..default()
        },
        PIXEL_PERFECT_LAYERS,
    ));

    let mut star = commands.spawn((
        Name::new("Star"),
        Star {
            active: false,
            speed: random_range(0.3..=0.6),
        },
        Sprite {
            image: asset_server.load("sprites/stars/star1.png"),
            ..default()
        },
        Transform {
            translation: Vec3::new(-1000.0, -1000.0, -10.0),
            ..default()
        },
        DespawnOnExit(Screen::Game),
        PIXEL_PERFECT_LAYERS,
    ));
    star_pool.inactive_stars.push(star.id());

    for _ in 0..600 {
        let star = star.clone_and_spawn();
        star_pool.inactive_stars.push(star.id());
    }
}

fn splash_update(mut state: ResMut<NextState<Screen>>, time: Res<Time>, mut timer: Local<Timer>) {
    if timer.duration().is_zero() {
        timer.set_duration(std::time::Duration::from_secs(25));
        timer.reset();
    }

    timer.tick(time.delta());

    if timer.is_finished() {
        state.set(Screen::Title);
    }
}

fn spawn_stars(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut star_pool: ResMut<StarPool>,
) {
    let x = random_range(-428.0..428.0);
    let y = random_range(240.0..480.0);
    let variant = random_range(1..=2);

    let brightness: f32 = random_range(0.6..1.0);
    let brightness = brightness * brightness;

    let color = if random_range(0.0..1.0) < 0.5 {
        Color::linear_rgb(brightness, brightness * 0.9, brightness * 0.7) // warmer star
    } else {
        Color::linear_rgb(brightness * 0.8, brightness * 0.9, brightness) // cooler star
    };

    let star_entity = star_pool.inactive_stars.pop();

    if let Some(entity) = star_entity {
        commands.entity(entity).insert((
            Transform {
                translation: Vec3::new(x, y, -10.0),
                ..default()
            },
            Sprite {
                image: asset_server.load(format!("sprites/stars/star{variant}.png")),
                color,
                ..default()
            },
            Star {
                active: true,
                speed: random_range(0.3..=0.6),
            },
        ));
        star_pool.active_count += 1;
    } else if star_pool.active_count < 600 {
        spawn_new_star(&mut commands, &asset_server, x, y, variant, color);
        star_pool.active_count += 1;
    }
}

fn spawn_new_star(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    x: f32,
    y: f32,
    variant: u8,
    color: Color,
) {
    commands.spawn((
        Name::new("Star"),
        Star {
            active: true,
            speed: random_range(0.3..=0.6),
        },
        Sprite {
            image: asset_server.load(format!("sprites/stars/star{variant}.png")),
            color,
            ..default()
        },
        Transform {
            translation: Vec3::new(x, y, -10.0),
            ..default()
        },
        DespawnOnExit(Screen::Game),
        PIXEL_PERFECT_LAYERS,
    ));
}

fn update_stars(mut stars: Query<(&mut Transform, &mut Star)>) {
    for (mut transform, mut star) in stars.iter_mut() {
        transform.translation -= Vec3::Y * star.speed;
        if transform.translation.y < -240.0 {
            transform.translation.y = 240.0 + random_range(0.0..20.0);
            transform.translation.x = random_range(-400.0..400.0);
            star.speed = random_range(0.25..=0.5);
        }
    }
}

fn update_developer(
    mut commands: Commands,
    mut developer_transform: Query<(Entity, &mut Transform), With<Developer>>,
    time: Res<Time>,
    mut timer: Local<Timer>,
    mut dropping: Local<bool>,
) {
    if timer.duration().is_zero() {
        timer.set_duration(std::time::Duration::from_secs_f32(3.75));
        timer.reset();
        *dropping = true;
    } else if timer.is_finished() && *dropping {
        for (entity, mut transform) in developer_transform.iter_mut() {
            transform.translation.y -= 0.5;
            if transform.translation.y <= 0.0 {
                transform.translation.y = 0.0;
                *dropping = false;
                commands.entity(entity).insert(FadeOut);
            }
        }
    }
    timer.tick(time.delta());
}

fn update_title(
    mut title_transform: Query<&mut Transform, With<Title>>,
    time: Res<Time>,
    mut timer: Local<Timer>,
    mut dropping: Local<bool>,
) {
    if timer.duration().is_zero() {
        timer.set_duration(std::time::Duration::from_secs_f32(16.0));
        timer.reset();
        *dropping = true;
    } else if timer.is_finished() && *dropping {
        for mut transform in title_transform.iter_mut() {
            transform.translation.y -= 0.5;
            if transform.translation.y <= 32.0 {
                transform.translation.y = 32.0;
                *dropping = false;
            }
        }
    }
    timer.tick(time.delta());
}

fn fade_out(
    mut commands: Commands,
    mut fade_entities: Query<(Entity, &mut Sprite, &FadeOut)>,
    time: Res<Time>,
) {
    for (entity, mut sprite, _) in fade_entities.iter_mut() {
        let mut color = sprite.color;
        let alpha = color.alpha() - time.delta_secs() * 0.25;
        if alpha < 0.0 {
            commands.entity(entity).despawn();
            continue;
        } else {
            color.set_alpha(alpha);
            sprite.color = color;
        }
    }
}
