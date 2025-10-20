use bevy::{prelude::*, text::FontSmoothing};
use leafwing_input_manager::prelude::ActionState;

use crate::{PIXEL_PERFECT_LAYERS, ScaleFactor, input::Action, screens::Screen};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Credits), spawn_credits);
    app.add_systems(
        Update,
        (credits_input, credits_button).run_if(in_state(Screen::Credits)),
    );
}

fn spawn_credits(
    mut commands: Commands,
    scale_factor: Res<ScaleFactor>,
    asset_server: Res<AssetServer>,
) {
    let scale = scale_factor.0;
    let font_handle = asset_server.load("font.ttf");
    commands.spawn((
        Name::new("Credits container"),
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
        DespawnOnExit(Screen::Credits),
        PIXEL_PERFECT_LAYERS,
        children![
            (
                Name::new("Credits"),
                Text::new("CREDITS"),
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
                Text::new("MADE WITH LOVE AND BEVY FOR LDG GAME JAM 3"),
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
                Name::new("TITLE THEME"),
                Text::new("TITLE THEME - OPENGAMEART.ORG SPACE-THEME"),
                TextFont {
                    font_size: 8.0 * scale,
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
                Name::new("LEVEL THEME"),
                Text::new("LEVEL THEME - OPENGAMEART.ORG THROUGH-SPACE"),
                TextFont {
                    font_size: 8.0 * scale,
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
                Name::new("SFX"),
                Text::new("SFX - OPENGAMEART.ORG 512-SOUND-EFFECTS-8-BIT-STYLE"),
                TextFont {
                    font_size: 8.0 * scale,
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
                Name::new("EVERYTHING ELSE"),
                Text::new("EVERYTHING ELSE - JODA"),
                TextFont {
                    font_size: 8.0 * scale,
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

fn credits_button(
    interaction_query: Query<&Interaction, Changed<Interaction>>,
    mut screen: ResMut<NextState<Screen>>,
) {
    for interaction in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            screen.set(Screen::Title)
        }
    }
}

fn credits_input(
    action_state: Single<&ActionState<Action>>,
    mut screen: ResMut<NextState<Screen>>,
) {
    if action_state.just_pressed(&Action::Select) {
        screen.set(Screen::Title);
    }
}
