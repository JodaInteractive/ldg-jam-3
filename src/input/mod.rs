use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<Action>::default());
        app.add_systems(Startup, setup_input);
    }
}

#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
pub enum Action {
    Up,
    Down,
    Left,
    Right,
    Select,
    Shoot,
    Pause,
}

fn setup_input(mut commands: Commands) {
    use leafwing_input_manager::prelude::*;

    let mut player_input_map = InputMap::default();
    player_input_map.insert(Action::Up, GamepadControlDirection::LEFT_UP.threshold(0.25));
    player_input_map.insert(
        Action::Down,
        GamepadControlDirection::LEFT_DOWN.threshold(0.25),
    );
    player_input_map.insert(
        Action::Left,
        GamepadControlDirection::LEFT_LEFT.threshold(0.25),
    );
    player_input_map.insert(
        Action::Right,
        GamepadControlDirection::LEFT_RIGHT.threshold(0.25),
    );
    player_input_map.insert(Action::Up, KeyCode::KeyW);
    player_input_map.insert(Action::Up, KeyCode::ArrowUp);
    player_input_map.insert(Action::Up, GamepadButton::DPadUp);
    player_input_map.insert(Action::Down, KeyCode::KeyS);
    player_input_map.insert(Action::Down, KeyCode::ArrowDown);
    player_input_map.insert(Action::Down, GamepadButton::DPadDown);
    player_input_map.insert(Action::Left, KeyCode::KeyA);
    player_input_map.insert(Action::Left, KeyCode::ArrowLeft);
    player_input_map.insert(Action::Left, GamepadButton::DPadLeft);
    player_input_map.insert(Action::Right, KeyCode::KeyD);
    player_input_map.insert(Action::Right, KeyCode::ArrowRight);
    player_input_map.insert(Action::Right, GamepadButton::DPadRight);
    player_input_map.insert(Action::Select, KeyCode::Space);
    player_input_map.insert(Action::Shoot, KeyCode::Space);
    player_input_map.insert(Action::Shoot, GamepadButton::South);
    player_input_map.insert(Action::Select, KeyCode::Enter);
    player_input_map.insert(Action::Select, GamepadButton::South);
    player_input_map.insert(Action::Pause, KeyCode::Escape);

    commands.spawn(player_input_map);
}
