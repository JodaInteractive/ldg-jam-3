use bevy::{prelude::*, text::FontSmoothing};
use leafwing_input_manager::prelude::ActionState;

use crate::{
    PIXEL_PERFECT_LAYERS, ScaleFactor,
    input::Action,
    screens::{Screen, splash::Title},
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

fn spawn_title_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut title_transform: Query<&mut Transform, With<Title>>,
    scale_factor: Res<ScaleFactor>,
) {
    let scale = scale_factor.0;
    let mut transform = title_transform.single_mut().unwrap();
    transform.translation = Vec3::new(0.0, 32.0, 0.0);
    let font_handle: Handle<Font> = asset_server.load("font.ttf");

    let new_game = title_menu_button(
        "NEW GAME",
        0,
        MenuButton::NewGame,
        font_handle.clone(),
        scale,
    );
    let settings = title_menu_button(
        "SETTINGS",
        1,
        MenuButton::Settings,
        font_handle.clone(),
        scale,
    );

    #[cfg(target_arch = "wasm32")]
    let menu_buttons = children![new_game, settings];
    #[cfg(not(target_arch = "wasm32"))]
    let quit = title_menu_button("QUIT", 2, MenuButton::Quit, font_handle.clone(), scale);
    #[cfg(not(target_arch = "wasm32"))]
    let menu_buttons = children![new_game, settings, quit];

    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            padding: UiRect::all(Val::Px(8.0 * scale)),
            ..default()
        },
        DespawnOnExit(Screen::Title),
        PIXEL_PERFECT_LAYERS,
        children![(
            Name::new("Title Screen Menu"),
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                position_type: PositionType::Absolute,
                bottom: Val::Px(8.0 * scale),
                ..default()
            },
            PIXEL_PERFECT_LAYERS,
            menu_buttons,
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

    #[cfg(not(target_arch = "wasm32"))]
    let max_index = 2;
    #[cfg(target_arch = "wasm32")]
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

fn title_menu_button(
    button: &str,
    button_index: usize,
    menu_button: MenuButton,
    font_handle: Handle<Font>,
    scale: f32,
) -> impl Bundle {
    (
        Name::new(format!("{button} Button")),
        Button,
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(32.0 * scale),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        menu_button,
        PIXEL_PERFECT_LAYERS,
        children![(
            Name::new(format!("{button}Text")),
            Text::new(button.to_string()),
            TextColor(TRANSPARENT_MEDIUM_GRAY),
            ButtonText(button_index),
            TextFont {
                font_size: 16.0 * scale,
                font: font_handle,
                font_smoothing: FontSmoothing::None,
                ..default()
            },
            FadeIn,
        )],
    )
}
