use std::time::Duration;

use bevy::prelude::*;

use crate::{AppSystems, PausableSystems, game::game_state::GameState};

const BASE_ANIMATION_SPEED: u64 = 250;

#[derive(Clone, Component)]
pub struct Animation {
    current_frame: usize,
    frames: Vec<usize>,
    minigame_key: Option<&'static str>,
    timer: Timer,
}

impl Animation {
    pub fn new(frames: &[usize]) -> Self {
        Self {
            current_frame: 0,
            frames: frames.iter().copied().collect(),
            minigame_key: None,
            timer: Timer::new(
                Duration::from_millis(BASE_ANIMATION_SPEED),
                TimerMode::Repeating,
            ),
        }
    }

    pub fn with_minigame(mut self, minigame_key: &'static str) -> Self {
        self.minigame_key = Some(minigame_key);

        self
    }

    pub fn with_speed(mut self, ms: u64) -> Self {
        self.timer.set_duration(Duration::from_millis(ms));

        self
    }
}

fn animation_timer(
    animation_query: Query<(&mut Animation, &mut Sprite)>,
    game_state: Res<State<GameState>>,
    time: Res<Time>,
) {
    for (mut animation, mut sprite) in animation_query {
        if let Some(minigame_key) = animation.minigame_key {
            if game_state.get() != &GameState::Minigame(minigame_key.to_string()) {
                continue;
            }
        }

        animation.timer.tick(time.delta());

        if animation.timer.just_finished() {
            let Some(texture_atlas) = sprite.texture_atlas.as_mut() else {
                return;
            };

            animation.current_frame += 1;

            if animation.current_frame >= animation.frames.len() {
                animation.current_frame = 0;
            }

            texture_atlas.index = animation.frames[animation.current_frame];
        }
    }
}

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        animation_timer
            .in_set(AppSystems::TickTimers)
            .in_set(PausableSystems),
    );
}
