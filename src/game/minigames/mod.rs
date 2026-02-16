pub mod control;
pub mod observe;
pub mod relieve;

use bevy::prelude::*;

pub const MINIGAME_KEYS: [&'static str; 3] = [
    control::MINIGAME_KEY,
    observe::MINIGAME_KEY,
    relieve::MINIGAME_KEY,
];

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((control::plugin, observe::plugin, relieve::plugin));
}

pub(super) fn should_lose_on_timeout(key: &'static str) -> bool {
    match key {
        control::MINIGAME_KEY => control::SHOULD_LOSE_ON_TIMEOUT,
        observe::MINIGAME_KEY => observe::SHOULD_LOSE_ON_TIMEOUT,
        relieve::MINIGAME_KEY => relieve::SHOULD_LOSE_ON_TIMEOUT,
        _ => true,
    }
}

pub(super) fn spawn_minigame(key: &str, commands: &mut EntityCommands) {
    match key {
        control::MINIGAME_KEY => commands.insert(control::spawn_minigame()),
        observe::MINIGAME_KEY => commands.insert(observe::spawn_minigame()),
        relieve::MINIGAME_KEY => commands.insert(relieve::spawn_minigame()),
        _ => panic!("No minigame with key: {}", key),
    };
}
