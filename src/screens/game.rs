use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::{input::Action, screens::Screen};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Game), spawn_game_screen);
    app.add_systems(Update, game_loop.run_if(in_state(Screen::Game)));
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
        children![(
            Name::new("Player Hitbox"),
            Sprite {
                image: asset_server.load("sprites/ships/ship1-hitbox.png"),
                ..default()
            }
        )],
    ));
}

#[derive(Component)]
struct Player;

fn game_loop(
    input_query: Query<&ActionState<Action>>,
    mut player: Query<&mut Transform, With<Player>>,
) {
    let action_state = input_query.single().unwrap();
    let mut player_transform = player.single_mut().unwrap();

    if action_state.pressed(&Action::Up) {
        println!("Up pressed");
        player_transform.translation += Vec3::Y * 5.0;
    }

    if action_state.pressed(&Action::Down) {
        println!("Down pressed");
        player_transform.translation -= Vec3::Y * 5.0;
    }

    if action_state.pressed(&Action::Left) {
        println!("Left pressed");
        player_transform.translation -= Vec3::X * 5.0;
    }

    if action_state.pressed(&Action::Right) {
        println!("Right pressed");
        player_transform.translation += Vec3::X * 5.0;
    }
}
