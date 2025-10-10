use bevy::prelude::*;

use crate::screens::Screen;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Title), spawn_title_screen)
        .add_systems(Update, button_system);
}

#[derive(Component)]
enum MenuButton {
    NewGame,
}

fn spawn_title_screen(mut commands: Commands) {
    commands.spawn((
        Node { ..default() },
        DespawnOnExit(Screen::Title),
        children![(
            Name::new("Title Screen Menu"),
            Node {
                width: Val::Percent(70.0),
                max_width: Val::Px(900.0),
                margin: UiRect::all(Val::Auto),
                ..default()
            },
            children![
                (
                    Name::new("Title"),
                    Text::new("Star Journey"),
                    Node {
                        margin: UiRect::all(Val::Auto),
                        ..default()
                    },
                ),
                (
                    Name::new("Play"),
                    Button,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(100.0),
                        ..default()
                    },
                    MenuButton::NewGame,
                    children![(Name::new("Play Text"), Text::new("New Game"),)]
                )
            ]
        )],
    ));
}

fn button_system(
    interaction_query: Query<(&Interaction, &MenuButton), Changed<Interaction>>,
    mut screen_state: ResMut<NextState<Screen>>,
) {
    for (interaction, menu_button) in interaction_query {
        match *interaction {
            Interaction::Pressed => match menu_button {
                MenuButton::NewGame => {
                    screen_state.set(Screen::Game);
                }
            },
            Interaction::Hovered => {}
            Interaction::None => {}
        }
    }
}
