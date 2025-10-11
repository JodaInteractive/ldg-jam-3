use bevy::{
    asset::AssetMetaCheck,
    prelude::*,
    remote::{RemotePlugin, http::RemoteHttpPlugin},
    window::{WindowMode, WindowResolution},
};
use bevy_embedded_assets::{EmbeddedAssetPlugin, PluginMode};

use crate::input::InputPlugin;

mod input;
mod screens;

fn main() -> AppExit {
    App::new().add_plugins(AppPlugin).run()
}

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EmbeddedAssetPlugin {
            mode: PluginMode::ReplaceDefault,
        });

        app.add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Star Journey".to_string(),
                        fit_canvas_to_parent: true,
                        mode: WindowMode::Windowed,
                        resolution: WindowResolution::new(1920, 1080),
                        resizable: false,
                        position: WindowPosition::Centered(MonitorSelection::Current),
                        ..default()
                    }),
                    ..default()
                }),
        );

        app.add_plugins(InputPlugin);

        app.add_plugins(RemotePlugin::default());
        app.add_plugins(RemoteHttpPlugin::default());

        app.add_plugins(screens::plugin);

        app.init_state::<Pause>();
        app.configure_sets(Update, PausableSystems.run_if(in_state(Pause(false))));

        app.insert_resource(ClearColor(Color::srgb_u8(49, 54, 56)));
        app.add_systems(Startup, spawn_camera);
    }
}

#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
struct Pause(pub bool);

#[derive(SystemSet, Copy, Clone, Eq, PartialEq, Hash, Debug)]
struct PausableSystems;

fn spawn_camera(mut commands: Commands) {
    commands.spawn((Name::new("Camera"), Camera2d));
}
