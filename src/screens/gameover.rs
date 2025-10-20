use bevy::{prelude::*, text::FontSmoothing};
use leafwing_input_manager::prelude::ActionState;

use crate::{
    PIXEL_PERFECT_LAYERS, ScaleFactor,
    input::Action,
    screens::{
        Screen,
        game::{GameState, GameStats, GameSystems, RestartGame},
    },
    sundry::{BLACK, MEDIUM_GRAY, WHITE},
};

pub(super) fn plugin(app: &mut App) {
    app.insert_resource(GameOverActiveIndex(0));
    app.configure_sets(Startup, GameSystems::GameOver);

    app.add_systems(OnEnter(GameState::GameOver), spawn_game_over);
    app.add_systems(
        Update,
        (game_over_buttons, game_over_input_system)
            .in_set(GameSystems::GameOver)
            .run_if(in_state(Screen::Game).and(in_state(GameState::GameOver))),
    );
    app.add_systems(
        Update,
        style_button_text.run_if(in_state(Screen::Game).and(in_state(GameState::GameOver))),
    );
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
                    padding: UiRect::all(Val::Px(8.0 * scale)),
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
                Name::new("Game Over Button Container"),
                Node {
                    position_type: PositionType::Absolute,
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    bottom: Val::Px(0.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                children![
                    game_over_button(
                        "REPLAY",
                        0,
                        GameOverButton::Replay,
                        font_handle.clone(),
                        scale
                    ),
                    game_over_button(
                        "BACK TO MENU",
                        1,
                        GameOverButton::BackToMenu,
                        font_handle.clone(),
                        scale
                    )
                ]
            )
        ],
    ));
}

fn game_over_button(
    button: &str,
    index: usize,
    game_over_button: GameOverButton,
    font_handle: Handle<Font>,
    scale: f32,
) -> impl Bundle {
    (
        Name::new(format!("{button} Button")),
        game_over_button,
        Node::DEFAULT,
        children![(
            Name::new(format!("{button} Button Text")),
            Text::new(button),
            Node::default(),
            GameOverButtonIndex(index),
            TextFont {
                font_size: 16.0 * scale,
                font: font_handle.clone(),
                font_smoothing: FontSmoothing::None,
                ..default()
            }
        )],
    )
}

#[derive(Component)]
#[require(Button)]
enum GameOverButton {
    Replay,
    BackToMenu,
}

#[derive(Component)]
struct GameOverButtonIndex(usize);

#[derive(Resource)]
struct GameOverActiveIndex(usize);

fn game_over_input_system(
    mut commands: Commands,
    input_query: Query<&ActionState<Action>>,
    mut screen_state: ResMut<NextState<Screen>>,
    mut active_index: ResMut<GameOverActiveIndex>,
) {
    let action_state = input_query.single().unwrap();

    let max_index = 1;

    if action_state.just_pressed(&Action::Up) {
        if active_index.0 == 0 {
            active_index.0 = max_index;
        } else {
            active_index.0 -= 1;
        }
    }
    if action_state.just_pressed(&Action::Down) {
        if active_index.0 == max_index {
            active_index.0 = 0
        } else {
            active_index.0 += 1
        };
    }

    if action_state.just_pressed(&Action::Select) {
        if active_index.0 == 0 {
            commands.trigger(RestartGame);
        } else if active_index.0 == 1 {
            commands.trigger(RestartGame);
            screen_state.set(Screen::Title);
        }
    }
}

fn game_over_buttons(
    mut commands: Commands,
    interaction_query: Query<(&Interaction, &GameOverButton), Changed<Interaction>>,
    mut screen_state: ResMut<NextState<Screen>>,
    mut active_index: ResMut<GameOverActiveIndex>,
) {
    for (interaction, game_button) in interaction_query {
        match *interaction {
            Interaction::Pressed => match game_button {
                GameOverButton::Replay => commands.trigger(RestartGame),
                GameOverButton::BackToMenu => {
                    commands.trigger(RestartGame);
                    screen_state.set(Screen::Title);
                }
            },
            Interaction::Hovered => match game_button {
                GameOverButton::Replay => active_index.0 = 0,
                GameOverButton::BackToMenu => active_index.0 = 1,
            },
            _ => {}
        }
    }
}

fn style_button_text(
    mut text: Query<(&GameOverButtonIndex, &mut TextColor)>,
    active: ResMut<GameOverActiveIndex>,
) {
    for (button_index, mut color) in text.iter_mut() {
        if button_index.0 == active.0 {
            color.0 = WHITE;
        } else {
            color.0 = MEDIUM_GRAY;
        }
    }
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
