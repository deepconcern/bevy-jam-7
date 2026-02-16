use std::time::Duration;

use bevy::prelude::*;
use rand::seq::IndexedRandom;

use crate::{
    AppSystems, PausableSystems,
    game::{events::NewMinigame, game_state::GameState, minigames::MINIGAME_KEYS},
};

const WAIT_TIME: u64 = 3000;

#[derive(Resource)]
pub struct MinigameManager {
    pub current_minigame_key: Option<&'static str>,
    pub wait_timer: Timer,
}

impl Default for MinigameManager {
    fn default() -> Self {
        Self {
            current_minigame_key: None,
            wait_timer: Timer::new(Duration::from_millis(WAIT_TIME), TimerMode::Once),
        }
    }
}

impl MinigameManager {
    fn tick(
        mut commands: Commands,
        mut minigame_manager: ResMut<MinigameManager>,
        time: Res<Time>,
    ) {
        minigame_manager.wait_timer.tick(time.delta());

        if minigame_manager.wait_timer.just_finished() {
            let mut rng = rand::rng();

            minigame_manager.current_minigame_key = Some(MINIGAME_KEYS.choose(&mut rng).unwrap());
            minigame_manager.wait_timer.reset();
            commands.trigger(NewMinigame);
        }
    }
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<MinigameManager>();

    app.add_systems(
        Update,
        MinigameManager::tick
            .in_set(AppSystems::TickTimers)
            .in_set(PausableSystems)
            .run_if(in_state(GameState::Interlude)),
    );
}
