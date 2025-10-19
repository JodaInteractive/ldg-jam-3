use bevy::{
    audio::{PlaybackMode, Volume},
    prelude::*,
};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Soundtracks>();
    app.init_resource::<SfxLibrary>();
    app.add_observer(on_play_soundtrack_event);
    app.add_observer(on_play_sfx_event);
    app.add_systems(Update, fade_in);
}

#[derive(Resource)]
pub struct SfxLibrary {
    pub shoot: Handle<AudioSource>,
    pub explosion: Handle<AudioSource>,
}

impl FromWorld for SfxLibrary {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let sfx = SfxLibrary {
            shoot: asset_server.load("sfx/shooting.wav"),
            explosion: asset_server.load("sfx/explosion.wav"),
        };
        Self {
            shoot: sfx.shoot,
            explosion: sfx.explosion,
        }
    }
}

pub enum SoundEffect {
    Shoot,
    Explosion,
}

#[derive(Event)]
pub struct PlaySfxEvent {
    pub sfx: SoundEffect,
}

#[derive(Component)]
struct SoundtrackPlayer;

#[derive(Resource)]
pub struct Soundtracks {
    pub main_theme: Handle<AudioSource>,
    pub battle_theme: Handle<AudioSource>,
}

impl FromWorld for Soundtracks {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let soundtracks = Soundtracks {
            main_theme: asset_server.load("soundtrack/spacetheme.ogg"),
            battle_theme: asset_server.load("soundtrack/through-space.ogg"),
        };
        Self {
            main_theme: soundtracks.main_theme,
            battle_theme: soundtracks.battle_theme,
        }
    }
}

pub enum Soundtrack {
    MainTheme,
    BattleTheme,
}

#[derive(Event)]
pub struct PlaySoundtrackEvent {
    pub soundtrack: Soundtrack,
}

fn on_play_soundtrack_event(
    soundtrack_event: On<PlaySoundtrackEvent>,
    mut commands: Commands,
    soundtracks: Res<Soundtracks>,
    mut audio_player: Query<
        (Entity, &mut AudioPlayer, &mut PlaybackSettings),
        With<SoundtrackPlayer>,
    >,
) {
    let track_handle = match soundtrack_event.soundtrack {
        Soundtrack::MainTheme => soundtracks.main_theme.clone(),
        Soundtrack::BattleTheme => soundtracks.battle_theme.clone(),
    };

    let Ok((entity, mut audio, mut playback)) = audio_player.single_mut() else {
        commands.spawn((
            SoundtrackPlayer,
            AudioPlayer(track_handle.clone()),
            PlaybackSettings {
                mode: PlaybackMode::Loop,
                volume: Volume::Linear(0.0),
                ..default()
            },
            FadeIn { duration: 4.0 },
            Transform::default(),
            GlobalTransform::default(),
        ));
        return;
    };

    if audio.0 != track_handle {
        audio.0 = track_handle.clone();
        playback.volume = Volume::Linear(0.0);
        commands.entity(entity).insert(FadeIn { duration: 4.0 });
    }
}

fn on_play_sfx_event(sfx_event: On<PlaySfxEvent>, mut commands: Commands, sfx: Res<SfxLibrary>) {
    let sfx_handle = match sfx_event.sfx {
        SoundEffect::Shoot => sfx.shoot.clone(),
        SoundEffect::Explosion => sfx.explosion.clone(),
    };

    commands.spawn((
        AudioPlayer(sfx_handle),
        PlaybackSettings {
            mode: PlaybackMode::Remove,
            volume: Volume::Linear(0.1),
            ..default()
        },
        Transform::default(),
        GlobalTransform::default(),
    ));
}

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
