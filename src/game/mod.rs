mod animation;
mod events;
pub mod game_assets;
mod game_state;
mod minigame_manager;
mod minigame_timer;
mod minigames;
mod transition;
mod ui;

use bevy::prelude::*;

use crate::{
    AppSystems, PausableSystems, app_is_loaded,
    game::{
        events::{
            MinigameFinished, MinigameSpawned, NewMinigame, ResultsSpawned, SpawnMinigame,
            SpawnResults,
        },
        game_assets::GameAssets,
        game_state::GameState,
        minigame_manager::MinigameManager,
        minigame_timer::MinigameTimer,
        transition::Transition,
        ui::{GAME_OVER_FEVER, NO_FEVER, Thermometer, thermometer},
    },
    screens::Screen,
};

const FEVER_INCREMENT: f32 = 0.5;
const PLAYER_X: f32 = -18.0;
const PLAYER_Y: f32 = 4.0;

// Sizes
pub const MAIN_STAGE_HEIGHT: f32 = 9.0 * 16.0;
pub const MAIN_STAGE_WIDTH: f32 = 12.0 * 16.0;
const UI_HEIGHT: f32 = 9.0 * 16.0;
const UI_WIDTH: f32 = 4.0 * 16.0;

pub const MAIN_STAGE_TRANSFORM: Transform = Transform::from_xyz(
    64.0 + (MAIN_STAGE_WIDTH / 2.0),
    MAIN_STAGE_HEIGHT / 2.0,
    0.0,
);

#[derive(Component)]
struct MainStage;

#[derive(Component)]
struct Minigame;

#[derive(Component)]
#[require(Sprite, Transform)]
struct Player;

impl Player {
    fn added(game_assets: Res<GameAssets>, player_query: Query<&mut Sprite, Added<Player>>) {
        for mut sprite in player_query {
            sprite.image = game_assets.player.clone();
            sprite.texture_atlas = Some(TextureAtlas {
                layout: game_assets.player_layout.clone(),
                index: 0,
            });
        }
    }

    fn new() -> impl Bundle {
        (Player, Transform::from_xyz(PLAYER_X, PLAYER_Y, 10.0))
    }

    fn render(
        mut player_query: Query<&mut Sprite, With<Player>>,
        thermometer_query: Query<&Thermometer>,
    ) {
        for thermometer in thermometer_query {
            for mut sprite in player_query.iter_mut() {
                let Some(texture_atlas) = sprite.texture_atlas.as_mut() else {
                    continue;
                };

                texture_atlas.index = if thermometer.is_fever_high() {
                    2
                } else if thermometer.is_fever_low() {
                    1
                } else {
                    0
                };
            }
        }
    }
}

fn test_new_minigame(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    mut minigame_manager: ResMut<MinigameManager>,
) {
    if input.just_pressed(KeyCode::Space) {
        minigame_manager.current_minigame_key = Some(minigames::control::MINIGAME_KEY);
        commands.trigger(NewMinigame);
    }
}

pub fn in_minigame(
    minigame_key: &'static str,
) -> impl Fn(Res<State<GameState>>, Res<MinigameManager>) -> bool {
    move |game_state: Res<State<GameState>>, minigame_manager: Res<MinigameManager>| -> bool {
        game_state.get() == &GameState::Minigame
            && Some(minigame_key) == minigame_manager.current_minigame_key
    }
}

pub(super) fn plugin(app: &mut App) {
    app.init_state::<GameState>();

    app.add_observer(
        |trigger: On<MinigameFinished>,
         mut commands: Commands,
         mut next_state: ResMut<NextState<GameState>>| {
            commands.spawn(Transition::fade_out(trigger.0));
            next_state.set(GameState::Transitioning);
        },
    );
    app.add_observer(
        |_: On<NewMinigame>,
         mut commands: Commands,
         mut next_state: ResMut<NextState<GameState>>| {
            commands.spawn(Transition::fade_in());
            next_state.set(GameState::Transitioning);
        },
    );
    app.add_observer(
        |_: On<SpawnMinigame>,
         mut commands: Commands,
         main_stage_query: Query<Entity, With<MainStage>>,
         minigame_manager: Res<MinigameManager>| {
            let Ok(main_stage_entity) = main_stage_query.single() else {
                return;
            };

            let minigame_entity = commands
                .spawn((
                    Minigame,
                    Name::new(format!(
                        "Minigame \"{}\"",
                        minigame_manager.current_minigame_key.unwrap()
                    )),
                    Transform::from_xyz(0.0, 0.0, 10.0),
                ))
                .id();

            minigames::spawn_minigame(
                minigame_manager.current_minigame_key.unwrap(),
                &mut commands.entity(minigame_entity),
            );

            commands
                .entity(main_stage_entity)
                .add_child(minigame_entity);

            commands.trigger(MinigameSpawned);
        },
    );
    app.add_observer(
        |trigger: On<SpawnResults>,
         mut commands: Commands,
         game_assets: Res<GameAssets>,
         main_stage_query: Query<Entity, With<MainStage>>,
         minigame_query: Query<Entity, With<Minigame>>,
         player_query: Query<Entity, With<Player>>,
         mut thermometer_query: Query<&mut Thermometer>| {
            let Ok(main_stage_entity) = main_stage_query.single() else {
                return;
            };

            let Ok(minigame_entity) = minigame_query.single() else {
                return;
            };

            let Ok(mut thermometer) = thermometer_query.single_mut() else {
                return;
            };

            if trigger.0 {
                thermometer.reading -= FEVER_INCREMENT;
            } else {
                thermometer.reading += FEVER_INCREMENT;
            }

            // Check for game finished
            let (is_game_finished, has_won) = if thermometer.reading <= NO_FEVER {
                (true, true)
            } else if thermometer.reading >= GAME_OVER_FEVER {
                (true, false)
            } else {
                (false, false)
            };

            commands.entity(minigame_entity).despawn();

            if is_game_finished {
                let finished_screen_entity = commands
                    .spawn(Sprite::from_image(if has_won {
                        game_assets.win_screen.clone()
                    } else {
                        game_assets.lose_screen.clone()
                    }))
                    .id();

                for player_entity in player_query {
                    commands.entity(player_entity).despawn();
                }

                commands
                    .entity(main_stage_entity)
                    .add_child(finished_screen_entity);

                commands.spawn((
                    crate::theme::widget::ui_root("", true),
                    DespawnOnExit(Screen::Gameplay),
                    children![crate::theme::widget::button(
                        "To Title",
                        crate::menus::pause::quit_to_title
                    ),],
                    Transform::from_xyz(0.0, 0.0, 11.0),
                ));
            }

            commands.trigger(ResultsSpawned(is_game_finished));
        },
    );
    app.add_plugins((
        animation::plugin,
        game_assets::plugin,
        minigame_manager::plugin,
        minigame_timer::plugin,
        minigames::plugin,
        transition::plugin,
        ui::plugin,
    ));
    app.add_systems(
        Update,
        (
            Player::added.in_set(AppSystems::Update),
            (
                test_new_minigame.in_set(AppSystems::RecordInput),
                Player::render.in_set(AppSystems::Update),
            )
                .run_if(in_state(GameState::Interlude))
                .in_set(PausableSystems),
        )
            .run_if(app_is_loaded),
    );
}

pub fn spawn_game(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    // Init game state
    next_state.set(GameState::Interlude);

    // Core game
    commands.spawn((
        DespawnOnExit(Screen::Gameplay),
        MainStage,
        MAIN_STAGE_TRANSFORM.clone(),
        Name::new("Game"),
        Sprite::from_image(game_assets.interlude_background.clone()),
        Visibility::default(),
        children![Player::new()],
    ));

    // Game UI
    let ui_font = TextFont {
        font: game_assets.font.clone(),
        font_size: 8.0,
        ..default()
    };
    let ui_text_layout = TextLayout::new_with_justify(Justify::Center);
    let ui_text_background = TextBackgroundColor::BLACK;

    commands.spawn((
        DespawnOnExit(Screen::Gameplay),
        Name::new("Game UI"),
        Sprite::from_image(game_assets.ui_background.clone()),
        Transform::from_xyz(UI_WIDTH / 2.0, UI_HEIGHT / 2.0, 50.0),
        children![
            (
                Text2d::new("TIMER"),
                Transform::from_xyz(0.0, (UI_HEIGHT / 2.0) - 16.0, 1.0)
                    .with_scale(Vec3::splat(0.5)),
                ui_font.clone(),
                ui_text_background.clone(),
                ui_text_layout.clone(),
                children![MinigameTimer::new()],
            ),
            (
                Text2d::new("FEVER"),
                Transform::from_xyz(0.0, (UI_HEIGHT / 2.0) - 48.0, 1.0)
                    .with_scale(Vec3::splat(0.5)),
                ui_font.clone(),
                ui_text_background.clone(),
                ui_text_layout.clone(),
                children![thermometer(game_assets)],
            ),
            (
                Text2d::new("CONTROLS\n\nWASD: Movement\nSPACE: Action"),
                Transform::from_xyz(0.0, (UI_HEIGHT / 2.0) - 112.0, 1.0)
                    .with_scale(Vec3::splat(0.5)),
                ui_font.clone(),
                ui_text_background.clone(),
                ui_text_layout.clone(),
            ),
        ],
    ));
}
