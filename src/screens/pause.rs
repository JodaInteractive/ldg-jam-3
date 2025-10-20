use bevy::{prelude::*, text::FontSmoothing};
use leafwing_input_manager::prelude::ActionState;

use crate::{
    PIXEL_PERFECT_LAYERS, ScaleFactor,
    input::Action,
    screens::{
        Screen,
        game::{BackToMenuEvent, GameState, GameSystems},
    },
    sundry::{MEDIUM_GRAY, WHITE},
};

pub(super) fn plugin(app: &mut App) {
    app.init_state::<PauseState>();
    app.insert_resource(PauseActiveIndex(0));
    app.add_observer(pause);
    app.add_observer(resume);
    app.add_systems(
        Update,
        (pause_button_system, pause_input_system)
            .run_if(in_state(Screen::Game).and(in_state(PauseState::Paused))),
    );
    app.add_systems(
        Update,
        (pause_input)
            .in_set(GameSystems::Play)
            .run_if(in_state(Screen::Game).and(in_state(GameState::Play))),
    );
    app.add_systems(
        Update,
        style_button_text.run_if(in_state(Screen::Game).and(in_state(PauseState::Paused))),
    );
}

#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub enum PauseState {
    #[default]
    NotPaused,
    Paused,
}

#[derive(Component)]
#[require(Button)]
enum PauseButton {
    Resume,
    Menu,
}

#[derive(Component)]
struct PauseButtonIndex(usize);

#[derive(Resource)]
struct PauseActiveIndex(usize);

fn pause_input(
    mut commands: Commands,
    time: ResMut<Time<Virtual>>,
    action_state: Single<&ActionState<Action>>,
) {
    if action_state.just_pressed(&Action::Pause) {
        if time.is_paused() {
            commands.trigger(ResumeEvent);
        } else {
            commands.trigger(PauseEvent);
        }
    }
}

#[derive(Event)]
struct PauseEvent;

fn pause(
    _event: On<PauseEvent>,
    commands: Commands,
    mut time: ResMut<Time<Virtual>>,
    asset_server: Res<AssetServer>,
    scale_factor: Res<ScaleFactor>,
    mut pause_state: ResMut<NextState<PauseState>>,
    mut pause_active_index: ResMut<PauseActiveIndex>,
) {
    pause_active_index.0 = 0;
    time.pause();
    pause_state.set(PauseState::Paused);
    spawn_pause_menu(commands, asset_server.load("font.ttf"), scale_factor.0);
}

#[derive(Event)]
struct ResumeEvent;

fn resume(
    _event: On<ResumeEvent>,
    mut time: ResMut<Time<Virtual>>,
    mut pause_state: ResMut<NextState<PauseState>>,
) {
    time.unpause();
    pause_state.set(PauseState::NotPaused);
}

fn pause_menu_button(
    button: &str,
    index: usize,
    pause_button: PauseButton,
    font_handle: Handle<Font>,
    scale: f32,
) -> impl Bundle {
    (
        Name::new(format!("{button} Button")),
        pause_button,
        children![(
            Name::new(format!("{button} Button Text")),
            Text::new(button),
            PauseButtonIndex(index),
            TextColor(WHITE),
            TextFont {
                font_size: 16.0 * scale,
                font: font_handle,
                font_smoothing: FontSmoothing::None,
                ..default()
            },
            Node {
                padding: UiRect::all(Val::Px(8.0 * scale)),
                ..default()
            }
        )],
    )
}

fn spawn_pause_menu(mut commands: Commands, font: Handle<Font>, scale: f32) {
    commands.spawn((
        Name::new("Pause Menu"),
        Node {
            position_type: PositionType::Absolute,
            margin: UiRect::all(Val::Auto),
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        DespawnOnExit(PauseState::Paused),
        PIXEL_PERFECT_LAYERS,
        children![
            pause_menu_button("RESUME", 0, PauseButton::Resume, font.clone(), scale),
            pause_menu_button("BACK TO MENU", 1, PauseButton::Menu, font.clone(), scale),
        ],
    ));
}

fn pause_input_system(
    mut commands: Commands,
    input_query: Query<&ActionState<Action>>,
    mut active_index: ResMut<PauseActiveIndex>,
) {
    let action_state: &ActionState<Action> = input_query.single().unwrap();

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
            commands.trigger(ResumeEvent);
        } else if active_index.0 == 1 {
            commands.trigger(BackToMenuEvent);
        }
    }
}

fn pause_button_system(
    mut commands: Commands,
    interaction_query: Query<(&Interaction, &PauseButton), Changed<Interaction>>,
    mut active_index: ResMut<PauseActiveIndex>,
) {
    for (interaction, pause_button) in interaction_query {
        match *interaction {
            Interaction::Pressed => match pause_button {
                PauseButton::Resume => commands.trigger(ResumeEvent),
                PauseButton::Menu => commands.trigger(BackToMenuEvent),
            },
            Interaction::Hovered => match pause_button {
                PauseButton::Resume => active_index.0 = 0,
                PauseButton::Menu => active_index.0 = 1,
            },
            _ => {}
        }
    }
}

fn style_button_text(
    mut text: Query<(&PauseButtonIndex, &mut TextColor)>,
    active: ResMut<PauseActiveIndex>,
) {
    for (button_index, mut color) in text.iter_mut() {
        if button_index.0 == active.0 {
            color.0 = WHITE;
        } else {
            color.0 = MEDIUM_GRAY;
        }
    }
}
