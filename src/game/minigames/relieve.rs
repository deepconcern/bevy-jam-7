use std::time::Duration;

use bevy::{
    math::bounding::{Aabb2d, IntersectsVolume},
    prelude::*,
};
use bevy_asset_loader::prelude::*;

use crate::{
    AppSystems, PausableSystems, app_is_loaded,
    game::{events::MinigameFinished, in_minigame},
    screens::Screen,
};

pub const MINIGAME_KEY: &'static str = "relieve";

const DROP_BOUNDING_BOX: Vec2 = Vec2::new(3.0, 4.0);
const DROP_LIFETIME: u64 = 2000;
const DROP_MOVEMENT_SPEED: f32 = 100.0;
const DROP_Y: f32 = 39.0;
const HAND_MOVEMENT_SPEED: f32 = 50.0;
const HAND_Y: f32 = 56.0;
const FLOWER_BOUNDING_BOX: Vec2 = Vec2::new(5.0, 23.0);
const FLOWER_FRAMES: [usize; 3] = [0, 1, 2];
const FLOWER_HP: usize = 3;
const FLOWER_Y: f32 = -32.0;
const MOVEABLE_HORIZONTAL_BOUNDRY: f32 = 75.0;

#[derive(Component)]
#[require(Sprite, Transform)]
struct Flower {
    has_bloomed: bool,
    hp: usize,
}

impl Flower {
    fn added(flower_query: Query<&mut Sprite, Added<Flower>>, relieve_assets: Res<RelieveAssets>) {
        for mut sprite in flower_query {
            sprite.image = relieve_assets.flower.clone();
            sprite.texture_atlas = Some(TextureAtlas {
                layout: relieve_assets.flower_layout.clone(),
                index: 0,
            });
        }
    }

    fn check_hp(flower_query: Query<&mut Flower>) {
        for mut flower in flower_query {
            if flower.hp == 0 {
                flower.has_bloomed = true;
            }
        }
    }

    fn new(x: f32) -> impl Bundle {
        (
            Flower {
                has_bloomed: false,
                hp: FLOWER_HP,
            },
            Transform::from_xyz(x, FLOWER_Y, 10.0),
        )
    }

    fn render(flower_query: Query<(&Flower, &mut Sprite)>) {
        for (flower, mut sprite) in flower_query {
            let Some(texture_atlas) = sprite.texture_atlas.as_mut() else {
                continue;
            };

            texture_atlas.index = if flower.has_bloomed {
                *FLOWER_FRAMES.last().unwrap()
            } else {
                *FLOWER_FRAMES.first().unwrap()
            }
        }
    }
}

#[derive(Component)]
#[require(Sprite, Transform)]
struct Drop {
    lifetime: Timer,
}

impl Drop {
    fn added(drop_query: Query<&mut Sprite, Added<Drop>>, relieve_assets: Res<RelieveAssets>) {
        for mut sprite in drop_query {
            sprite.image = relieve_assets.drop.clone();
        }
    }

    fn lifetime(mut commands: Commands, drop_query: Query<(&mut Drop, Entity)>, time: Res<Time>) {
        for (mut drop, entity) in drop_query {
            drop.lifetime.tick(time.delta());

            if drop.lifetime.just_finished() {
                commands.entity(entity).despawn();
            }
        }
    }

    fn movement(drop_query: Query<&mut Transform, With<Drop>>, time: Res<Time>) {
        for mut transform in drop_query {
            transform.translation += Vec3::NEG_Y * DROP_MOVEMENT_SPEED * time.delta_secs();
        }
    }

    fn new(x: f32) -> impl Bundle {
        (
            Drop {
                lifetime: Timer::new(Duration::from_millis(DROP_LIFETIME), TimerMode::Once),
            },
            Transform::from_xyz(x, DROP_Y, 10.0),
        )
    }
}

#[derive(Component)]
#[require(Sprite, Transform)]
struct Hand;

impl Hand {
    fn added(hand_query: Query<&mut Sprite, Added<Hand>>, relieve_assets: Res<RelieveAssets>) {
        for mut sprite in hand_query {
            sprite.image = relieve_assets.hand.clone();
            sprite.texture_atlas = Some(TextureAtlas {
                layout: relieve_assets.hand_layout.clone(),
                index: 0,
            });
        }
    }

    fn movement(
        mut hand_query: Query<&mut Transform, With<Hand>>,
        input: Res<ButtonInput<KeyCode>>,
        time: Res<Time>,
    ) {
        let Ok(mut transform) = hand_query.single_mut() else {
            return;
        };

        let mut input_direction = Vec2::ZERO.clone();

        if input.pressed(KeyCode::KeyA) || input.pressed(KeyCode::ArrowLeft) {
            input_direction.x -= 1.0;
        }
        if input.pressed(KeyCode::KeyD) || input.pressed(KeyCode::ArrowRight) {
            input_direction.x += 1.0;
        }

        input_direction = input_direction.normalize_or_zero();

        let mut new_translation = transform.translation
            + input_direction.extend(0.0) * HAND_MOVEMENT_SPEED * time.delta_secs();

        // Boundry checking
        new_translation.x = new_translation.x.min(MOVEABLE_HORIZONTAL_BOUNDRY);
        new_translation.x = new_translation.x.max(-MOVEABLE_HORIZONTAL_BOUNDRY);

        transform.translation = new_translation;
    }

    fn new() -> impl Bundle {
        (Hand, Transform::from_xyz(0.0, HAND_Y, 10.0))
    }
}

#[derive(AssetCollection, Resource)]
pub struct RelieveAssets {
    #[asset(path = "images/relieve_background.png")]
    pub background: Handle<Image>,
    #[asset(path = "images/relieve_drop.png")]
    pub drop: Handle<Image>,
    #[asset(path = "images/relieve_flower.png")]
    pub flower: Handle<Image>,
    #[asset(texture_atlas_layout(tile_size_x = 32, tile_size_y = 64, columns = 3, rows = 1))]
    pub flower_layout: Handle<TextureAtlasLayout>,
    #[asset(path = "images/relieve_hand.png")]
    pub hand: Handle<Image>,
    #[asset(texture_atlas_layout(tile_size_x = 32, tile_size_y = 32, columns = 2, rows = 1))]
    pub hand_layout: Handle<TextureAtlasLayout>,
}

#[derive(Component)]
#[require(Sprite, Transform)]
struct Stage;

impl Stage {
    fn added(relieve_assets: Res<RelieveAssets>, stage_query: Query<&mut Sprite, Added<Stage>>) {
        for mut sprite in stage_query {
            sprite.image = relieve_assets.background.clone();
        }
    }
}

fn check_collisions(
    mut commands: Commands,
    drop_query: Query<(Entity, &Transform), With<Drop>>,
    mut flower_query: Query<(&mut Flower, &Transform), With<Flower>>,
) {
    for (drop_entity, drop_transform) in drop_query {
        let drop_aabb2d = Aabb2d::new(drop_transform.translation.truncate(), DROP_BOUNDING_BOX);

        for (mut flower, flower_transform) in flower_query.iter_mut() {
            if flower.has_bloomed {
                continue;
            }

            let flower_aabb2d =
                Aabb2d::new(flower_transform.translation.truncate(), FLOWER_BOUNDING_BOX);

            if drop_aabb2d.intersects(&flower_aabb2d) {
                commands.entity(drop_entity).despawn();
                flower.hp -= 1;
            }
        }
    }
}

fn check_win(mut commands: Commands, flower_query: Query<&Flower>) {
    let mut has_all_bloomed = true;

    for flower in flower_query {
        if !flower.has_bloomed {
            has_all_bloomed = false;

            break;
        }
    }

    if has_all_bloomed {
        commands.trigger(MinigameFinished(true));
    }
}

fn spawn_drop(
    mut commands: Commands,
    hand_query: Query<&Transform, With<Hand>>,
    input: Res<ButtonInput<KeyCode>>,
    stage_query: Query<Entity, With<Stage>>,
) {
    let Ok(transform) = hand_query.single() else {
        return;
    };

    let Ok(stage_entity) = stage_query.single() else {
        return;
    };

    if input.just_pressed(KeyCode::Space) {
        let drop_entity = commands.spawn(Drop::new(transform.translation.x)).id();
        commands.entity(stage_entity).add_child(drop_entity);
    }
}

pub(super) fn plugin(app: &mut App) {
    app.configure_loading_state(
        LoadingStateConfig::new(Screen::Loading).load_collection::<RelieveAssets>(),
    );

    app.add_systems(
        Update,
        (
            (Drop::added, Hand::added, Flower::added, Stage::added).in_set(AppSystems::Update),
            (
                (
                    check_collisions,
                    check_win,
                    Flower::check_hp,
                    Flower::render,
                )
                    .in_set(AppSystems::Update),
                (Drop::movement, Hand::movement, spawn_drop).in_set(AppSystems::RecordInput),
                Drop::lifetime.in_set(AppSystems::TickTimers),
            )
                .run_if(in_minigame(MINIGAME_KEY))
                .in_set(PausableSystems),
        )
            .run_if(app_is_loaded),
    );
}

pub(super) fn spawn_minigame() -> impl Bundle {
    (Stage, children![Hand::new(), Flower::new(0.0)])
}
