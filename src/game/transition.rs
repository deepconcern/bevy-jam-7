use std::time::Duration;

use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

use crate::{
    AppSystems, PausableSystems, app_is_loaded,
    game::{
        MAIN_STAGE_HEIGHT, MAIN_STAGE_WIDTH,
        events::{
            InterludeStart, MinigameSpawned, MinigameStart, ResultsSpawned, SpawnMinigame,
            SpawnResults,
        },
        game_assets::GameAssets,
        game_state::GameState,
        minigame_manager::MinigameManager,
    },
    screens::Screen,
};

const ANIMATION_SPEED: u64 = 100;
const FADE_IN_END_FRAME: usize = 14;
const FADE_IN_PAUSE_FRAME: usize = 7;
const FADE_IN_START_FRAME: usize = 0;

#[derive(Clone)]
pub enum TransitionState {
    // Start fade-in (from start to pause)
    FadeA,
    // Finish fade-in (from pause to end) (true if game finished)
    FadeB(bool),
    // Spawn things and show text
    Paused,
}

#[derive(Component)]
pub struct TransitionText;

#[derive(Clone, Eq, PartialEq)]
pub enum TransitionType {
    FadeIn,
    FadeOut(bool),
}

#[derive(Component)]
#[require(Sprite, Transform)]
pub struct Transition {
    pub state: TransitionState,
    pub timer: Timer,
    pub transition_type: TransitionType,
}

impl Transition {
    fn added(
        game_assets: Res<GameAssets>,
        minigame_manager: Res<MinigameManager>,
        mut text_font_query: Query<(&mut Text2d, &mut TextFont)>,
        transition_assets: Res<TransitionAssets>,
        transition_query: Query<(&Children, &mut Sprite, &Transition), Added<Transition>>,
    ) {
        for (children, mut sprite, transition) in transition_query {
            let start_frame = match &transition.transition_type {
                TransitionType::FadeIn => FADE_IN_START_FRAME,
                TransitionType::FadeOut(_) => FADE_IN_END_FRAME,
            };

            let Some(child) = children.first() else {
                continue;
            };

            let Ok((mut text_2d, mut text_font)) = text_font_query.get_mut(*child) else {
                continue;
            };

            sprite.image = transition_assets.transition.clone();
            sprite.texture_atlas = Some(TextureAtlas {
                layout: transition_assets.transition_layout.clone(),
                index: start_frame,
            });
            if transition.transition_type == TransitionType::FadeIn {
                text_2d.0 = minigame_manager
                    .current_minigame_key
                    .unwrap_or("minigame")
                    .to_uppercase();
            }
            text_font.font = game_assets.font.clone();
        }
    }

    fn new(transition_type: TransitionType) -> impl Bundle {
        let text = match &transition_type {
            TransitionType::FadeIn => "MINIGAME".to_string(),
            TransitionType::FadeOut(has_won) => {
                if *has_won {
                    "WIN!\nFEVER\nDOWN".to_string()
                } else {
                    "LOSE!\nFEVER\nUP".to_string()
                }
            }
        };

        (
            DespawnOnExit(Screen::Gameplay),
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
                TransitionText,
            )],
        )
    }

    pub fn fade_in() -> impl Bundle {
        Self::new(TransitionType::FadeIn)
    }

    pub fn fade_out(has_won: bool) -> impl Bundle {
        Self::new(TransitionType::FadeOut(has_won))
    }
}

#[derive(AssetCollection, Resource)]
pub struct TransitionAssets {
    #[asset(path = "images/dream_transition.png")]
    pub transition: Handle<Image>,
    #[asset(texture_atlas_layout(tile_size_x = 192, tile_size_y = 144, columns = 4, rows = 4))]
    pub transition_layout: Handle<TextureAtlasLayout>,
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
        TransitionType::FadeIn => (FADE_IN_END_FRAME, texture_atlas.index + 1),
        TransitionType::FadeOut(_) => (FADE_IN_START_FRAME, texture_atlas.index - 1),
    };

    match transition.state.clone() {
        TransitionState::FadeA => {
            texture_atlas.index = next_index;

            if texture_atlas.index == FADE_IN_PAUSE_FRAME {
                transition.state = TransitionState::Paused;
                match &transition.transition_type {
                    TransitionType::FadeIn => {
                        commands.trigger(SpawnMinigame);
                    }
                    TransitionType::FadeOut(has_won) => {
                        commands.trigger(SpawnResults(*has_won));
                    }
                }
            } else {
                transition.timer.reset();
            }
        }
        TransitionState::FadeB(is_game_finished) => {
            texture_atlas.index = next_index;

            if texture_atlas.index == end_frame {
                match &transition.transition_type {
                    TransitionType::FadeIn => {
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
    app.configure_loading_state(
        LoadingStateConfig::new(Screen::Loading).load_collection::<TransitionAssets>(),
    );

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
            Transition::added,
            (
                transition_timer.in_set(AppSystems::TickTimers),
                transition_animate.in_set(AppSystems::Update),
            )
                .in_set(PausableSystems),
        )
            .run_if(app_is_loaded),
    );
}
