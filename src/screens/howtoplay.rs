use bevy::{prelude::*, text::FontSmoothing};
use leafwing_input_manager::prelude::ActionState;

use crate::{PIXEL_PERFECT_LAYERS, ScaleFactor, input::Action, screens::Screen};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::HowToPlay), spawn_howtoplay);
    app.add_systems(
        Update,
        (howtoplay_input, howtoplay_button).run_if(in_state(Screen::HowToPlay)),
    );
}

fn spawn_howtoplay(
    mut commands: Commands,
    scale_factor: Res<ScaleFactor>,
    asset_server: Res<AssetServer>,
) {
    let scale = scale_factor.0;
    let font_handle = asset_server.load("font.ttf");
    commands.spawn((
        Name::new("HowToPlay container"),
        Node {
            width: Val::Percent(80.0),
            height: Val::Percent(80.0),
            margin: UiRect::all(Val::Auto),
            padding: UiRect::all(Val::Px(8.0 * scale)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        DespawnOnExit(Screen::HowToPlay),
        PIXEL_PERFECT_LAYERS,
        children![
            (
                Name::new("HowToPlay"),
                Text::new("HOW TO PLAY"),
                TextFont {
                    font_size: 16.0 * scale,
                    font: font_handle.clone(),
                    font_smoothing: FontSmoothing::None,
                    ..default()
                },
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(0.0),
                    margin: UiRect {
                        left: Val::Auto,
                        right: Val::Auto,
                        ..default()
                    },
                    ..default()
                }
            ),
            (
                Text::new("MOVE WITH WASD / ARROWS / ANALOG STICK / D PAD"),
                TextFont {
                    font_size: 12.0 * scale,
                    font: font_handle.clone(),
                    font_smoothing: FontSmoothing::None,
                    ..default()
                },
                Node {
                    padding: UiRect {
                        top: Val::Px(8.0 * scale),
                        bottom: Val::Px(8.0 * scale),
                        ..default()
                    },
                    ..default()
                },
            ),
            (
                Text::new("SHOOT WITH SPACE / BOTTOM BUTTON ON GAMEPAD CLUSTER"),
                TextFont {
                    font_size: 12.0 * scale,
                    font: font_handle.clone(),
                    font_smoothing: FontSmoothing::None,
                    ..default()
                },
                Node {
                    padding: UiRect {
                        top: Val::Px(8.0 * scale),
                        bottom: Val::Px(8.0 * scale),
                        ..default()
                    },
                    ..default()
                },
            ),
            (
                Text::new("PAUSE WITH ESCAPE / GAMEPAD START / GAMEPAD SELECT"),
                TextFont {
                    font_size: 12.0 * scale,
                    font: font_handle.clone(),
                    font_smoothing: FontSmoothing::None,
                    ..default()
                },
                Node {
                    padding: UiRect {
                        top: Val::Px(8.0 * scale),
                        bottom: Val::Px(8.0 * scale),
                        ..default()
                    },
                    ..default()
                },
            ),
            (
                Name::new("Back to menu button"),
                Button,
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(0.0),
                    margin: UiRect {
                        left: Val::Auto,
                        right: Val::Auto,
                        ..default()
                    },
                    ..default()
                },
                children![(
                    Name::new("Back to menu text"),
                    Text::new("BACK TO MENU"),
                    TextFont {
                        font_size: 16.0 * scale,
                        font: font_handle.clone(),
                        font_smoothing: FontSmoothing::None,
                        ..default()
                    }
                )]
            ),
        ],
    ));
}

fn howtoplay_button(
    interaction_query: Query<&Interaction, Changed<Interaction>>,
    mut screen: ResMut<NextState<Screen>>,
) {
    for interaction in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            screen.set(Screen::Title)
        }
    }
}

fn howtoplay_input(
    action_state: Single<&ActionState<Action>>,
    mut screen: ResMut<NextState<Screen>>,
) {
    if action_state.just_pressed(&Action::Select) {
        screen.set(Screen::Title);
    }
}
