use bevy::prelude::*;
use leafwing_input_manager::prelude::ActionState;

use crate::{
    HIGH_RES_LAYERS, PIXEL_PERFECT_LAYERS,
    input::Action,
    screens::Screen,
    sundry::{MEDIUM_GRAY, TRANSPARENT_MEDIUM_GRAY, WHITE},
};

pub(super) fn plugin(app: &mut App) {
    app.insert_resource(ActiveIndex(0));
    app.add_systems(OnEnter(Screen::Title), spawn_title_screen)
        .add_systems(
            Update,
            (button_system, input_system, fade_in).run_if(in_state(Screen::Title)),
        );
}

#[derive(Component)]
enum MenuButton {
    NewGame,
    Settings,
    Quit,
}

#[derive(Component)]
struct FadeIn;

#[derive(Resource)]
struct ActiveIndex(usize);

#[derive(Component)]
struct ButtonText(usize);

fn spawn_title_screen(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font_handle = asset_server.load("font.ttf");
    commands.spawn((
        Node {
            width: Val::Px(512.0),
            height: Val::Px(400.0),
            bottom: Val::Px(0.0),
            position_type: PositionType::Absolute,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            padding: UiRect::all(Val::Px(32.0)),
            margin: UiRect::all(Val::Auto),
            ..default()
        },
        DespawnOnExit(Screen::Title),
        children![(
            Name::new("Title Screen Menu"),
            Node {
                width: Val::Percent(70.0),
                max_width: Val::Px(900.0),
                margin: UiRect::all(Val::Auto),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            children![
                (
                    Name::new("New Game Button"),
                    Button,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(100.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    MenuButton::NewGame,
                    PIXEL_PERFECT_LAYERS,
                    children![(
                        Name::new("New Game Text"),
                        Text::new("NEW GAME"),
                        TextColor(TRANSPARENT_MEDIUM_GRAY),
                        ButtonText(0),
                        TextFont {
                            font_size: 32.0,
                            font: font_handle.clone(),
                            ..default()
                        },
                        FadeIn,
                    )]
                ),
                (
                    Name::new("Settings Button"),
                    Button,
                    MenuButton::Settings,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(100.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    HIGH_RES_LAYERS,
                    children![(
                        Name::new("Settings"),
                        Text::new("SETTINGS"),
                        TextColor(TRANSPARENT_MEDIUM_GRAY),
                        ButtonText(1),
                        TextFont {
                            font_size: 32.0,
                            font: font_handle.clone(),
                            ..default()
                        },
                        FadeIn,
                    )]
                ),
                (
                    Name::new("Quit Button"),
                    Button,
                    MenuButton::Quit,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(100.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    HIGH_RES_LAYERS,
                    children![(
                        Name::new("Quit Text"),
                        Text::new("QUIT"),
                        TextColor(TRANSPARENT_MEDIUM_GRAY),
                        ButtonText(2),
                        TextFont {
                            font_size: 32.0,
                            font: font_handle.clone(),
                            ..default()
                        },
                        FadeIn,
                    )]
                )
            ]
        )],
    ));
}

fn input_system(
    input_query: Query<&ActionState<Action>>,
    mut screen_state: ResMut<NextState<Screen>>,
    mut message_writer: MessageWriter<AppExit>,
    mut active_index: ResMut<ActiveIndex>,
    mut text_query: Query<(&ButtonText, &mut TextColor)>,
) {
    let action_state = input_query.single().unwrap();

    if action_state.just_pressed(&Action::Up) {
        if active_index.0 == 0 {
            active_index.0 = 2;
        } else {
            active_index.0 -= 1;
        }
    }

    if action_state.just_pressed(&Action::Down) {
        if active_index.0 == 2 {
            active_index.0 = 0
        } else {
            active_index.0 += 1
        };
    }

    for (button_index, mut text_color) in text_query.iter_mut() {
        if button_index.0 == active_index.0 {
            text_color.0 = WHITE;
        } else {
            text_color.0 = MEDIUM_GRAY;
        }
    }

    if action_state.just_pressed(&Action::Select) {
        match active_index.0 {
            0 => {
                screen_state.set(Screen::Game);
            }
            1 => {
                println!("Settings button pressed");
            }
            2 => {
                message_writer.write(AppExit::Success);
            }
            _ => {}
        }
    }
}

fn button_system(
    mut commands: Commands,
    interaction_query: Query<(&Interaction, &MenuButton, &Children), Changed<Interaction>>,
    mut screen_state: ResMut<NextState<Screen>>,
    mut message_writer: MessageWriter<AppExit>,
    mut active_index: ResMut<ActiveIndex>,
) {
    for (interaction, menu_button, children) in interaction_query {
        match *interaction {
            Interaction::Pressed => match menu_button {
                MenuButton::NewGame => {
                    screen_state.set(Screen::Game);
                }
                MenuButton::Settings => {
                    println!("Settings button pressed");
                }
                MenuButton::Quit => {
                    message_writer.write(AppExit::Success);
                }
            },
            Interaction::Hovered => {
                commands
                    .entity(*children.first().unwrap())
                    .insert(TextColor(WHITE));
                match menu_button {
                    MenuButton::NewGame => {
                        active_index.0 = 0;
                    }
                    MenuButton::Settings => {
                        active_index.0 = 1;
                    }
                    MenuButton::Quit => {
                        active_index.0 = 2;
                    }
                }
            }
            Interaction::None => {
                commands
                    .entity(*children.first().unwrap())
                    .insert(TextColor(MEDIUM_GRAY));
            }
        }
    }
}

fn fade_in(
    mut commands: Commands,
    mut query: Query<(&mut TextColor, Entity), With<FadeIn>>,
    time: Res<Time>,
) {
    for (mut ui_color, entity) in query.iter_mut() {
        let current_alpha = ui_color.0.alpha();
        let new_alpha = (current_alpha + time.delta_secs() / 2.0).min(1.0);
        ui_color.0.set_alpha(new_alpha);
        if new_alpha >= 1.0 {
            commands.entity(entity).remove::<FadeIn>();
        }
    }
}
