use bevy::prelude::*;

mod credits;
mod endgame;
mod game;
mod gameover;
mod pause;
mod splash;
mod title;

pub(super) fn plugin(app: &mut App) {
    app.init_state::<Screen>();

    app.add_plugins((
        splash::plugin,
        title::plugin,
        game::plugin,
        credits::plugin,
        pause::plugin,
        gameover::plugin,
        endgame::plugin,
    ));
}

#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub enum Screen {
    #[default]
    Splash,
    Title,
    Game,
    Credits,
}
