use bevy::{
    asset::AssetMetaCheck,
    camera::{RenderTarget, visibility::RenderLayers},
    prelude::*,
    render::render_resource::{
        Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    },
    window::{WindowMode, WindowResized, WindowResolution},
};

use bevy_embedded_assets::{EmbeddedAssetPlugin, PluginMode};

use crate::{input::InputPlugin, sundry::BLACK};

mod audio;
mod input;
mod screens;
mod stars;
mod sundry;

const RES_WIDTH: u32 = 853;
const RES_HEIGHT: u32 = 480;

const PIXEL_PERFECT_LAYERS: RenderLayers = RenderLayers::layer(0);
const HIGH_RES_LAYERS: RenderLayers = RenderLayers::layer(1);

#[derive(Component)]
struct Canvas;

#[derive(Component)]
struct InGameCamera;

#[derive(Component)]
struct OuterCamera;

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
                        #[cfg(not(debug_assertions))]
                        mode: WindowMode::BorderlessFullscreen(MonitorSelection::Current),
                        #[cfg(debug_assertions)]
                        mode: WindowMode::Windowed,
                        // resolution: WindowResolution::new(853, 480),
                        resolution: WindowResolution::new(1920, 1080),
                        resizable: false,
                        position: WindowPosition::Centered(MonitorSelection::Current),
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        );

        app.insert_resource(ScaleFactor(1.0));

        app.add_plugins(InputPlugin);

        app.add_plugins(audio::plugin);

        app.add_plugins(screens::plugin);

        app.add_plugins(stars::plugin);

        app.init_state::<Pause>();
        app.configure_sets(Update, PausableSystems.run_if(in_state(Pause(false))));

        app.insert_resource(ClearColor(Color::srgb_u8(9, 10, 20)));
        app.add_systems(PreStartup, spawn_camera);
        app.add_systems(Update, fit_canvas.run_if(on_message::<WindowResized>));
    }
}

#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
struct Pause(pub bool);

#[derive(SystemSet, Copy, Clone, Eq, PartialEq, Hash, Debug)]
struct PausableSystems;

fn spawn_camera(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let canvas_size = Extent3d {
        width: RES_WIDTH,
        height: RES_HEIGHT,
        ..default()
    };
    let mut canvas = Image {
        texture_descriptor: TextureDescriptor {
            label: Some("Canvas Texture"),
            size: canvas_size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::COPY_DST
                | TextureUsages::TEXTURE_BINDING
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };
    canvas.resize(canvas_size);
    let image_handle = images.add(canvas);
    commands.spawn((
        Camera2d,
        Camera {
            order: -1,
            target: RenderTarget::Image(image_handle.clone().into()),
            clear_color: ClearColorConfig::Custom(BLACK),
            ..default()
        },
        Msaa::Off,
        UiAntiAlias::Off,
        InGameCamera,
        PIXEL_PERFECT_LAYERS,
    ));

    commands.spawn((Sprite::from_image(image_handle), Canvas, HIGH_RES_LAYERS));
    commands.spawn((Camera2d, Msaa::Off, OuterCamera, HIGH_RES_LAYERS));
}

#[derive(Resource)]
pub struct ScaleFactor(f32);

fn fit_canvas(
    mut resize_messages: MessageReader<WindowResized>,
    mut projection: Single<&mut Projection, With<OuterCamera>>,
    mut scale_factor: ResMut<ScaleFactor>,
) {
    let Projection::Orthographic(proj) = &mut **projection else {
        return;
    };

    for msg in resize_messages.read() {
        let h_scale = msg.width / RES_WIDTH as f32;
        let v_scale = msg.height / RES_HEIGHT as f32;
        let scale = h_scale.min(v_scale);
        proj.scale = 1. / scale;
        scale_factor.0 = scale;
    }
}
