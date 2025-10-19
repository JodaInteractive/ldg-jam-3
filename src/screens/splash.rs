use bevy::prelude::*;
use leafwing_input_manager::prelude::ActionState;
use rand::random_range;

use crate::{
    PIXEL_PERFECT_LAYERS,
    audio::{PlaySoundtrackEvent, Soundtrack},
    input::Action,
    screens::Screen,
    stars::{Star, StarPool},
};

#[derive(Component)]
pub struct Title;

#[derive(Component)]
struct Developer;

#[derive(Component)]
struct FadeOut;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Splash), (setup_splash, trigger_music));
    app.add_systems(
        FixedUpdate,
        (
            splash_update,
            update_title,
            update_developer,
            fade_out,
            skip_button,
        )
            .run_if(in_state(Screen::Splash)),
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

fn trigger_music(mut commands: Commands) {
    commands.trigger(PlaySoundtrackEvent {
        soundtrack: Soundtrack::MainTheme,
    });
}

fn skip_button(mut state: ResMut<NextState<Screen>>, keyboard_input: Query<&ActionState<Action>>) {
    let keyboard_input = keyboard_input.single();
    if keyboard_input.is_err() {
        return;
    }

    let keyboard_input = keyboard_input.unwrap();
    if keyboard_input.just_pressed(&Action::Select) || keyboard_input.just_pressed(&Action::Pause) {
        state.set(Screen::Title);
    }
}
