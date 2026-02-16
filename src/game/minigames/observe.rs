use std::time::Duration;

use bevy::{
    math::bounding::{Aabb2d, IntersectsVolume},
    prelude::*,
};
use bevy_asset_loader::prelude::*;
use rand::Rng;

use crate::{
    AppSystems, PausableSystems, app_is_loaded,
    game::{
        MAIN_STAGE_HEIGHT, MAIN_STAGE_WIDTH, animation::Animation, events::MinigameFinished,
        game_assets::GameAssets, in_minigame,
    },
    screens::Screen,
};

pub const MINIGAME_KEY: &'static str = "observe";
pub const SHOULD_LOSE_ON_TIMEOUT: bool = true;

const COLLECT_AMOUNT: usize = 3;
const STAR_LIFETIME: u64 = 2000;

// Animation
const GALILEO_FRAMES: [usize; 2] = [0, 1];
const STAR_FRAMES: [usize; 2] = [0, 1];

// Speed
const GALILEO_MOVEMENT_SPEED: f32 = 75.0;
const STAR_MOVEMENT_SPEED: f32 = 100.0;
const STAR_SPAWN_SPEED: u64 = 500;

// Boundries
const STAR_BOUNDRY_BOX: Vec2 = Vec2::new(4.0, 8.0);
const TELESCOPE_BOUNDRY_BOX: Vec2 = Vec2::new(10.0, 4.0);
const TELESCOPE_BOUNDRY_OFFSET: f32 = 21.0;
const WALKABLE_HORIZONTAL_BOUNDRY: f32 = 75.0;
const WALKABLE_VERTICAL_BOUNDRY_BOTTOM: f32 = -47.0;
const WALKABLE_VERTICAL_BOUNDRY_TOP: f32 = -15.0;

#[derive(Component)]
struct Score;

impl Score {
    fn new() -> impl Bundle {
        (
            Score,
            Text2d::new(format!("0/{}", COLLECT_AMOUNT)),
            Transform::from_xyz(0.0, (MAIN_STAGE_HEIGHT / 2.0) - 10.0, 0.0),
        )
    }

    fn added(game_assets: Res<GameAssets>, score_query: Query<&mut TextFont, Added<Score>>) {
        for mut text_font in score_query {
            text_font.font_size = 10.0;
            text_font.font = game_assets.font.clone();
        }
    }

    fn render(observe_manager: Res<ObserveManager>, score_query: Query<&mut Text2d, With<Score>>) {
        for mut text in score_query {
            text.0 = format!("{}/{}", observe_manager.collected, COLLECT_AMOUNT);
        }
    }
}

#[derive(Component)]
#[require(Sprite, Transform)]
struct Stage {
    star_timer: Timer,
}

impl Default for Stage {
    fn default() -> Self {
        Self {
            star_timer: Timer::new(
                Duration::from_millis(STAR_SPAWN_SPEED),
                TimerMode::Repeating,
            ),
        }
    }
}

#[derive(Component)]
#[require(Sprite, Transform)]
struct Star {
    life_timer: Timer,
}

impl Star {
    fn new(observe_assets: Res<ObserveAssets>, origin: Vec2, target: Vec2) -> impl Bundle {
        let mut transform = Transform::from_translation(origin.extend(0.0));

        transform.rotation = Quat::from_rotation_z(Vec2::Y.angle_to(target));

        (
            Animation::looping(&STAR_FRAMES).with_minigame(MINIGAME_KEY),
            Sprite {
                image: observe_assets.star.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: observe_assets.star_layout.clone(),
                    index: STAR_FRAMES[0],
                }),
                flip_y: true,
                ..default()
            },
            Self {
                life_timer: Timer::new(Duration::from_millis(STAR_LIFETIME), TimerMode::Once),
            },
            transform,
        )
    }
}

#[derive(Component)]
#[require(Sprite, Transform)]
struct Galileo;

#[derive(AssetCollection, Clone, Reflect, Resource)]
#[reflect(Resource)]
pub struct ObserveAssets {
    #[asset(path = "images/observe_background.png")]
    pub background: Handle<Image>,
    #[asset(path = "images/observe_catch.png")]
    pub catch: Handle<Image>,
    #[asset(texture_atlas_layout(tile_size_x = 16, tile_size_y = 16, columns = 2, rows = 4))]
    pub catch_layout: Handle<TextureAtlasLayout>,
    #[asset(path = "images/observe_galileo.png")]
    pub galileo: Handle<Image>,
    #[asset(texture_atlas_layout(tile_size_x = 32, tile_size_y = 40, columns = 2, rows = 1))]
    pub galileo_layout: Handle<TextureAtlasLayout>,
    #[asset(path = "images/observe_star.png")]
    pub star: Handle<Image>,
    #[asset(texture_atlas_layout(tile_size_x = 8, tile_size_y = 16, columns = 2, rows = 1))]
    pub star_layout: Handle<TextureAtlasLayout>,
}

#[derive(Resource)]
struct ObserveManager {
    collected: usize,
}

impl Default for ObserveManager {
    fn default() -> Self {
        Self { collected: 0 }
    }
}

fn check_win(mut commands: Commands, observe_manager: Res<ObserveManager>) {
    if observe_manager.collected >= COLLECT_AMOUNT {
        commands.trigger(MinigameFinished(true));
    }
}

fn galileo_added(
    observe_assets: Res<ObserveAssets>,
    mut stage_query: Query<&mut Sprite, Added<Galileo>>,
) {
    let Ok(mut sprite) = stage_query.single_mut() else {
        return;
    };

    sprite.image = observe_assets.galileo.clone();
    sprite.texture_atlas = Some(TextureAtlas {
        index: 0,
        layout: observe_assets.galileo_layout.clone(),
    });
}

fn check_observed(
    mut commands: Commands,
    galileo_query: Query<&Transform, With<Galileo>>,
    mut observe_manager: ResMut<ObserveManager>,
    // stage_query: Query<Entity, With<Stage>>,
    star_query: Query<(Entity, &Transform), With<Star>>,
) {
    let Ok(galileo_transform) = galileo_query.single() else {
        return;
    };

    let galileo_aabb = Aabb2d::new(
        Vec2::new(
            galileo_transform.translation.x,
            galileo_transform.translation.y + TELESCOPE_BOUNDRY_OFFSET,
        ),
        TELESCOPE_BOUNDRY_BOX,
    );

    // let Ok(stage_entity) = stage_query.single() else {
    //     return;
    // };

    for (star_entity, star_transform) in star_query {
        let star_aabb = Aabb2d::new(star_transform.translation.truncate(), STAR_BOUNDRY_BOX);

        if galileo_aabb.intersects(&star_aabb) {
            commands.entity(star_entity).despawn();
            observe_manager.collected += 1;

            // TODO: Spawn effect
        }
    }
}

fn galileo_movement(
    mut galileo_query: Query<&mut Transform, With<Galileo>>,
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let Ok(mut transform) = galileo_query.single_mut() else {
        return;
    };

    let mut input_direction = Vec2::ZERO.clone();

    if input.pressed(KeyCode::KeyW) || input.pressed(KeyCode::ArrowUp) {
        input_direction.y += 1.0;
    }
    if input.pressed(KeyCode::KeyS) || input.pressed(KeyCode::ArrowDown) {
        input_direction.y -= 1.0;
    }
    if input.pressed(KeyCode::KeyA) || input.pressed(KeyCode::ArrowLeft) {
        input_direction.x -= 1.0;
    }
    if input.pressed(KeyCode::KeyD) || input.pressed(KeyCode::ArrowRight) {
        input_direction.x += 1.0;
    }

    input_direction = input_direction.normalize_or_zero();

    let mut new_translation = transform.translation
        + input_direction.extend(0.0) * GALILEO_MOVEMENT_SPEED * time.delta_secs();

    // Boundry checking
    new_translation.x = new_translation.x.min(WALKABLE_HORIZONTAL_BOUNDRY);
    new_translation.x = new_translation.x.max(-WALKABLE_HORIZONTAL_BOUNDRY);
    new_translation.y = new_translation.y.min(WALKABLE_VERTICAL_BOUNDRY_TOP);
    new_translation.y = new_translation.y.max(WALKABLE_VERTICAL_BOUNDRY_BOTTOM);

    transform.translation = new_translation;
}

fn spawn_stars(
    mut commands: Commands,
    observe_assets: Res<ObserveAssets>,
    mut stage_query: Query<(Entity, &mut Stage)>,
    time: Res<Time>,
) {
    let Ok((stage_entity, mut stage)) = stage_query.single_mut() else {
        return;
    };

    let mut rng = rand::rng();

    stage.star_timer.tick(time.delta());

    if stage.star_timer.just_finished() {
        let star_entity = commands
            .spawn(Star::new(
                observe_assets,
                Vec2::new(
                    (rng.random::<f32>() * MAIN_STAGE_WIDTH) - (MAIN_STAGE_WIDTH / 2.0),
                    MAIN_STAGE_HEIGHT / 2.0,
                ),
                Vec2::new(
                    (rng.random::<f32>() * MAIN_STAGE_WIDTH) - (MAIN_STAGE_WIDTH / 2.0),
                    -MAIN_STAGE_HEIGHT / 2.0,
                ),
            ))
            .id();

        commands.entity(stage_entity).add_child(star_entity);
    }
}

fn stage_added(
    observe_assets: Res<ObserveAssets>,
    mut observe_manager: ResMut<ObserveManager>,
    mut stage_query: Query<&mut Sprite, Added<Stage>>,
) {
    let Ok(mut sprite) = stage_query.single_mut() else {
        return;
    };

    *observe_manager = ObserveManager::default();

    sprite.image = observe_assets.background.clone();
}

fn star_lifetime(mut commands: Commands, star_query: Query<(Entity, &mut Star)>, time: Res<Time>) {
    for (star_entity, mut star) in star_query {
        star.life_timer.tick(time.delta());

        if star.life_timer.just_finished() {
            commands.entity(star_entity).despawn();
        }
    }
}

fn star_movement(star_query: Query<&mut Transform, With<Star>>, time: Res<Time>) {
    for mut transform in star_query {
        let direction = transform.rotation * Vec3::Y;

        transform.translation += direction * STAR_MOVEMENT_SPEED * time.delta_secs();
    }
}

pub fn spawn_minigame() -> impl Bundle {
    (
        Stage::default(),
        children![
            (
                Animation::looping(&GALILEO_FRAMES),
                Galileo,
                Transform::from_xyz(0.0, -24.0, 5.0)
            ),
            Score::new()
        ],
    )
}

pub fn plugin(app: &mut App) {
    app.configure_loading_state(
        LoadingStateConfig::new(Screen::Loading).load_collection::<ObserveAssets>(),
    );

    app.init_resource::<ObserveManager>();

    app.add_systems(
        Update,
        (
            (galileo_added, stage_added).in_set(AppSystems::Update),
            ((
                galileo_movement.in_set(AppSystems::RecordInput),
                (spawn_stars, star_lifetime).in_set(AppSystems::TickTimers),
                (
                    check_observed,
                    check_win,
                    Score::added,
                    Score::render,
                    star_movement,
                )
                    .in_set(AppSystems::Update),
            )
                .run_if(in_minigame(MINIGAME_KEY)),)
                .in_set(PausableSystems),
        )
            .run_if(app_is_loaded),
    );
}
