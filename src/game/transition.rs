use std::time::Duration;

use bevy::prelude::*;

use crate::{
    AppSystems, PausableSystems,
    asset_tracking::LoadResource,
    game::{
        MAIN_STAGE_HEIGHT, MAIN_STAGE_WIDTH,
        events::{
            InterludeStart, MinigameSpawned, MinigameStart, ResultsSpawned, SpawnMinigame,
            SpawnResults,
        },
        game_state::GameState,
    },
    screens::Screen,
};

const ANIMATION_SPEED: u64 = 100;
const FADE_IN_END_FRAME: usize = 14;
const FADE_IN_PAUSE_FRAME: usize = 7;
const FADE_IN_START_FRAME: usize = 0;
const TRANSITION_HEIGHT: f32 = 9.0 * 16.0;
const TRANSITION_WIDTH: f32 = 12.0 * 16.0;

#[derive(Clone)]
pub enum TransitionState {
    // Start fade-in (from start to pause)
    FadeA,
    // Finish fade-in (from pause to end) (true if game finished)
    FadeB(bool),
    // Spawn things and show text
    Paused,
}

#[derive(Clone)]
pub enum TransitionType {
    FadeIn(String),
    FadeOut(bool),
}

#[derive(Component)]
pub struct Transition {
    pub state: TransitionState,
    pub timer: Timer,
    pub transition_type: TransitionType,
}

impl Transition {
    fn new(
        transition_type: TransitionType,
        transition_assets: Res<TransitionAssets>,
    ) -> impl Bundle {
        let (start_frame, text) = match &transition_type {
            TransitionType::FadeIn(minigame) => (FADE_IN_START_FRAME, minigame.to_uppercase()),
            TransitionType::FadeOut(has_won) => (
                FADE_IN_END_FRAME,
                if *has_won {
                    "WIN!\nFEVER\nDOWN".to_string()
                } else {
                    "LOSE!\nFEVER\nUP".to_string()
                },
            ),
        };

        (
            DespawnOnExit(Screen::Gameplay),
            Sprite::from_atlas_image(
                transition_assets.transition.clone(),
                TextureAtlas {
                    layout: transition_assets.transition_layout.clone(),
                    index: start_frame,
                },
            ),
            Transform::from_xyz(
                64.0 + (MAIN_STAGE_WIDTH / 2.0),
                MAIN_STAGE_HEIGHT / 2.0,
                100.0,
            ),
            Self {
                transition_type,
                state: TransitionState::FadeA,
                timer: Timer::new(Duration::from_millis(ANIMATION_SPEED), TimerMode::Once),
            },
            children![(
                Text2d::new(text),
                TextLayout::new_with_justify(Justify::Center),
                TextShadow {
                    color: Color::WHITE,
                    offset: Vec2::new(5.0, 5.0),
                },
                TextColor(Color::BLACK),
                TextFont {
                    font: transition_assets.font.clone(),
                    ..default()
                },
            )],
        )
    }

    pub fn fade_in(minigame: &str, transition_assets: Res<TransitionAssets>) -> impl Bundle {
        Self::new(
            TransitionType::FadeIn(minigame.to_string()),
            transition_assets,
        )
    }

    pub fn fade_out(has_won: bool, transition_assets: Res<TransitionAssets>) -> impl Bundle {
        Self::new(TransitionType::FadeOut(has_won), transition_assets)
    }
}

#[derive(Asset, Clone, Reflect, Resource)]
#[reflect(Resource)]
pub struct TransitionAssets {
    pub font: Handle<Font>,
    pub transition: Handle<Image>,
    pub transition_layout: Handle<TextureAtlasLayout>,
}

impl FromWorld for TransitionAssets {
    fn from_world(world: &mut World) -> Self {
        let mut texture_atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();

        let transition_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
            UVec2::new(TRANSITION_WIDTH as u32, TRANSITION_HEIGHT as u32),
            4,
            4,
            None,
            None,
        ));

        let asset_server = world.resource::<AssetServer>();

        Self {
            font: asset_server.load("fonts/PressStart2P-Regular.ttf"),
            transition: asset_server.load("images/dream_transition.png"),
            transition_layout,
        }
    }
}

fn transition_animate(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    mut transition_query: Query<(Entity, &mut Sprite, &mut Transition)>,
) {
    let Ok((entity, mut sprite, mut transition)) = transition_query.single_mut() else {
        return;
    };

    if !transition.timer.just_finished() {
        return;
    }

    let Some(texture_atlas) = sprite.texture_atlas.as_mut() else {
        return;
    };

    let (end_frame, next_index) = match transition.transition_type {
        TransitionType::FadeIn(_) => (FADE_IN_END_FRAME, texture_atlas.index + 1),
        TransitionType::FadeOut(_) => (FADE_IN_START_FRAME, texture_atlas.index - 1),
    };

    match transition.state.clone() {
        TransitionState::FadeA => {
            texture_atlas.index = next_index;

            if texture_atlas.index == FADE_IN_PAUSE_FRAME {
                transition.state = TransitionState::Paused;
                match transition.transition_type.clone() {
                    TransitionType::FadeIn(minigame) => {
                        commands.trigger(SpawnMinigame(minigame.clone()));
                    }
                    TransitionType::FadeOut(has_won) => {
                        commands.trigger(SpawnResults(has_won));
                    }
                }
            } else {
                transition.timer.reset();
            }
        }
        TransitionState::FadeB(is_game_finished) => {
            texture_atlas.index = next_index;

            if texture_atlas.index == end_frame {
                match transition.transition_type {
                    TransitionType::FadeIn(_) => {
                        commands.trigger(MinigameStart);
                        next_state.set(GameState::Minigame);
                    }
                    TransitionType::FadeOut(_) => {
                        commands.trigger(InterludeStart);
                        if is_game_finished {
                            next_state.set(GameState::GameOver)
                        } else {
                            next_state.set(GameState::Interlude);
                        }
                    }
                }
                commands.entity(entity).despawn();
            } else {
                transition.timer.reset();
            }
        }
        _ => {}
    }
}

fn transition_timer(time: Res<Time>, mut transition_query: Query<&mut Transition>) {
    let Ok(mut transition) = transition_query.single_mut() else {
        return;
    };

    transition.timer.tick(time.delta());
}

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<TransitionAssets>();

    app.add_observer(
        |_: On<MinigameSpawned>, mut transition_query: Query<&mut Transition>| {
            let Ok(mut transition) = transition_query.single_mut() else {
                return;
            };

            transition.state = TransitionState::FadeB(false);
            transition.timer.reset();
        },
    );

    app.add_observer(
        |trigger: On<ResultsSpawned>, mut transition_query: Query<&mut Transition>| {
            let Ok(mut transition) = transition_query.single_mut() else {
                return;
            };

            transition.state = TransitionState::FadeB(trigger.0);
            transition.timer.reset();
        },
    );

    app.add_systems(
        Update,
        (
            transition_timer.in_set(AppSystems::TickTimers),
            transition_animate.in_set(AppSystems::Update),
        )
            .in_set(PausableSystems),
    );
}
