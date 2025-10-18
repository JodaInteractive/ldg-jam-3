use bevy::prelude::*;
use rand::random_range;

use crate::{PIXEL_PERFECT_LAYERS, screens::Screen};

pub(super) fn plugin(app: &mut App) {
    app.insert_resource::<StarPool>(StarPool {
        active_count: 0,
        inactive_stars: Vec::new(),
    });
    app.add_systems(FixedUpdate, (spawn_stars, update_stars));
}

#[derive(Resource)]
pub struct StarPool {
    pub active_count: usize,
    pub inactive_stars: Vec<Entity>,
}

#[derive(Component)]
pub struct Star {
    pub speed: f32,
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

fn update_stars(mut stars: Query<(&mut Transform, &mut Star)>, active_screen: Res<State<Screen>>) {
    for (mut transform, mut star) in stars.iter_mut() {
        let speed = match active_screen.get() {
            Screen::Game => star.speed * 2.0,
            _ => star.speed,
        };
        transform.translation -= Vec3::Y * speed;
        if transform.translation.y < -240.0 {
            transform.translation.y = 240.0 + random_range(0.0..20.0);
            transform.translation.x = random_range(-400.0..400.0);
            star.speed = random_range(0.25..=0.5);
        }
    }
}
