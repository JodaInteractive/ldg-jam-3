use bevy::{
    audio::{PlaybackMode, Volume},
    prelude::*,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, setup_audio);
    // app.add_observer(on_play_soundtrack_event);
    app.add_systems(Update, fade_in);
}

#[derive(Resource)]
pub struct Soundtracks {
    pub main_theme: Handle<AudioSource>,
    pub battle_theme: Handle<AudioSource>,
}

pub enum Soundtrack {
    MainTheme,
    BattleTheme,
}

#[derive(Event)]
pub struct PlaySoundtrackEvent {
    soundtrack: Soundtrack,
}

fn setup_audio(mut commands: Commands, asset_server: Res<AssetServer>) {
    let main_handle = asset_server.load("soundtrack/spacetheme.ogg");
    let soundtracks = Soundtracks {
        main_theme: main_handle.clone(),
        battle_theme: asset_server.load("audio/soundtrack/battle_theme.ogg"),
    };

    commands.insert_resource(soundtracks);

    commands.spawn((
        AudioPlayer(main_handle.clone()),
        PlaybackSettings {
            mode: PlaybackMode::Loop,
            volume: Volume::Linear(0.0),
            ..default()
        },
        FadeIn { duration: 4.0 },
        Transform::default(),
        GlobalTransform::default(),
    ));
}

// fn on_play_soundtrack_event(
//     soundtrack_event: On<PlaySoundtrackEvent>,
//     soundtracks: Res<Soundtracks>,
// ) {
// let track_handle = match soundtrack_event.soundtrack {
//     Soundtrack::MainTheme => soundtracks.main_theme.clone(),
//     Soundtrack::BattleTheme => soundtracks.battle_theme.clone(),
// };

// commands.spawn((
//     AudioPlayer(track_handle),
//     PlaybackSettings {
//         mode: PlaybackMode::Loop,
//         volume: Volume::Linear(1.0),
//         ..default()
//     },
// ));
// }

#[derive(Component)]
struct FadeIn {
    duration: f32,
}

fn fade_in(
    mut commands: Commands,
    mut audio_sink: Query<(&FadeIn, &mut AudioSink, Entity)>,
    time: Res<Time>,
) {
    for (fade_in, mut audio, entity) in audio_sink.iter_mut() {
        let current_volume = audio.volume();
        audio.set_volume(
            current_volume.fade_towards(Volume::Linear(1.0), time.delta_secs() / fade_in.duration),
        );
        if audio.volume().to_linear() >= 1.0 {
            audio.set_volume(Volume::Linear(1.0));
            commands.entity(entity).remove::<FadeIn>();
        }
    }
}
