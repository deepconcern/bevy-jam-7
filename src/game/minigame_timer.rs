use std::time::Duration;

use bevy::prelude::*;

use crate::{
    AppSystems, PausableSystems,
    game::{
        events::{MinigameFinished, MinigameStart},
        minigame_manager::MinigameManager,
        minigames,
    },
    screens::Screen,
};

const MINIGAME_TIMER_DURATION: u64 = 5000;
const MINIGAME_TIMER_HEIGHT: f32 = 16.0;
const MINIGAME_TIMER_OFFSET: f32 = -32.0;
const MINIGAME_TIMER_WIDTH: f32 = 56.0;

#[derive(Component)]
#[require(Sprite, Transform, Visibility)]
pub struct MinigameTimer {
    minigame_key: Option<&'static str>,
    timer: Timer,
}

impl MinigameTimer {
    fn render(minigame_timer_query: Query<(&MinigameTimer, &mut Sprite)>) {
        for (minigame_timer, mut sprite) in minigame_timer_query {
            let custom_size = sprite.custom_size.unwrap();

            let percentage_remaining = minigame_timer.timer.remaining().as_secs_f32()
                / Duration::from_millis(MINIGAME_TIMER_DURATION).as_secs_f32();

            let timer_width = MINIGAME_TIMER_WIDTH * percentage_remaining;

            sprite.custom_size = Some(Vec2::new(timer_width, custom_size.y));
        }
    }

    fn tick(
        mut commands: Commands,
        minigame_timer_query: Query<&mut MinigameTimer>,
        time: Res<Time>,
    ) {
        for mut minigame_timer in minigame_timer_query {
            let Some(minigame_key) = minigame_timer.minigame_key else {
                return;
            };

            minigame_timer.timer.tick(time.delta());

            if minigame_timer.timer.just_finished() {
                commands.trigger(MinigameFinished(!minigames::should_lose_on_timeout(
                    minigame_key,
                )));
            }
        }
    }

    pub fn new() -> impl Bundle {
        let mut timer = Timer::new(
            Duration::from_millis(MINIGAME_TIMER_DURATION),
            TimerMode::Once,
        );

        timer.pause();

        (
            MinigameTimer {
                minigame_key: None,
                timer,
            },
            Sprite::from_color(
                Color::srgb_u8(227, 81, 0),
                Vec2::new(MINIGAME_TIMER_WIDTH, MINIGAME_TIMER_HEIGHT),
            ),
            Transform::from_xyz(0.0, MINIGAME_TIMER_OFFSET, 0.0),
            Visibility::Hidden,
        )
    }
}

pub(super) fn plugin(app: &mut App) {
    app.add_observer(
        |_: On<MinigameFinished>,
         mut minigame_timer_query: Query<(&mut MinigameTimer, &mut Visibility)>|
         -> Result {
            let (mut minigame_timer, mut visibility) = minigame_timer_query.single_mut()?;

            minigame_timer.minigame_key = None;
            minigame_timer.timer.reset();

            *visibility = Visibility::Hidden;

            Ok(())
        },
    );

    app.add_observer(
        |_: On<MinigameStart>,
         minigame_manager: Res<MinigameManager>,
         minigame_timer_query: Query<(&mut MinigameTimer, &mut Visibility)>| {
            for (mut minigame_timer, mut visibility) in minigame_timer_query {
                minigame_timer.minigame_key = minigame_manager.current_minigame_key;
                minigame_timer.timer.unpause();
                *visibility = Visibility::Visible;
            }
        },
    );

    app.add_systems(
        Update,
        (
            MinigameTimer::tick.in_set(AppSystems::TickTimers),
            MinigameTimer::render.in_set(AppSystems::Update),
        )
            .run_if(in_state(Screen::Gameplay))
            .in_set(PausableSystems),
    );
}
