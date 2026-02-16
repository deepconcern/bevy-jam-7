use bevy::prelude::*;

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, States)]
pub enum GameState {
    GameOver,
    #[default]
    Interlude,
    Minigame,
    Transitioning,
}
