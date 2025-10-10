use bevy::prelude::*;

use crate::screens::Screen;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Game), spawn_game_screen);
}

fn spawn_game_screen(mut commands: Commands) {
    commands.spawn((
        Name::new("Game Screen"),
        Text::new("Game Screen"),
        DespawnOnExit(Screen::Game),
    ));

    commands.spawn((
        Name::new("Player Ship"),
        Sprite {
            // image: Handle::,
            ..default()
        },
    ));
}
