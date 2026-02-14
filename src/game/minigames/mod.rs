// pub mod demo;
pub mod observe;
pub mod relieve;

use bevy::prelude::*;

// pub const MINIGAME_KEYS: [&'static str; 2] = [observe::MINIGAME_KEY, relieve::MINIGAME_KEY];

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((observe::plugin, relieve::plugin));
    // app.add_plugins((observe::plugin, relieve::plugin));
    // app.add_plugins((demo::plugin, observe::plugin));
}

pub(super) fn spawn_minigame(key: &str, commands: &mut EntityCommands) {
    match key {
        // demo::MINIGAME_KEY => commands.insert(demo::spawn_minigame()),
        observe::MINIGAME_KEY => commands.insert(observe::spawn_minigame()),
        relieve::MINIGAME_KEY => commands.insert(relieve::spawn_minigame()),
        _ => panic!("No minigame with key: {}", key),
    };
}
