use std::time::Duration;

use bevy::prelude::*;

use crate::{
    AppSystems, PausableSystems,
    game::{game_state::GameState, minigame_manager::MinigameManager},
};

const BASE_ANIMATION_SPEED: u64 = 250;

#[derive(Clone, Eq, PartialEq)]
enum AnimationType {
    Looping,
    OneOff,
}

#[derive(Clone, Component)]
pub struct Animation {
    animation_type: AnimationType,
    current_frame: usize,
    frames: Vec<usize>,
    minigame_key: Option<&'static str>,
    timer: Timer,
}

impl Animation {
    pub fn looping(frames: &[usize]) -> Self {
        Self {
            current_frame: frames[0],
            frames: frames.iter().copied().collect(),
            ..default()
        }
    }

    pub fn with_minigame(mut self, minigame_key: &'static str) -> Self {
        self.minigame_key = Some(minigame_key);

        self
    }
}

impl Default for Animation {
    fn default() -> Self {
        Self {
            animation_type: AnimationType::Looping,
            current_frame: 0,
            frames: Vec::new(),
            minigame_key: None,
            timer: Timer::new(
                Duration::from_millis(BASE_ANIMATION_SPEED),
                TimerMode::Repeating,
            ),
        }
    }
}

fn animation_timer(
    animation_query: Query<(&mut Animation, Entity, &mut Sprite)>,
    mut commands: Commands,
    game_state: Res<State<GameState>>,
    minigame_manager: Res<MinigameManager>,
    time: Res<Time>,
) {
    for (mut animation, entity, mut sprite) in animation_query {
        if let Some(minigame_key) = animation.minigame_key {
            if game_state.get() != &GameState::Minigame {
                continue;
            }

            if Some(minigame_key) != minigame_manager.current_minigame_key {
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
                if animation.animation_type == AnimationType::OneOff {
                    commands.entity(entity).remove::<Animation>();
                    continue;
                }

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
