use std::time::Duration;

use bevy::{
    math::bounding::{Aabb2d, IntersectsVolume},
    prelude::*,
};
use bevy_asset_loader::prelude::*;
use rand::Rng;

use crate::{
    AppSystems, PausableSystems, app_is_loaded,
    game::{events::MinigameFinished, in_minigame},
    screens::Screen,
};

pub const MINIGAME_KEY: &'static str = "control";
pub const SHOULD_LOSE_ON_TIMEOUT: bool = false;

const AGGRESSIVE_BACK_SPEED: f32 = 1000.0;
const AGGRESSIVE_BOUNDING_BOX: Vec2 = Vec2::new(40.0, 48.0);
const AGGRESSIVE_FORWARD_ACCELERATION: f32 = 7.0;
const AGGRESSIVE_FORWARD_SPEED: f32 = 20.0;
const AGGRESSIVE_X: f32 = -40.0;
const AGGRESSIVE_Y: f32 = -16.0;
const MOVEABLE_HORIZONTAL_BOUNDRY: f32 = 75.0;
const SCARED_BOUNDING_BOX: Vec2 = Vec2::new(2.0, 48.0);
const SCARED_MOVEMENT_SPEED: f32 = 10.0;
const SCARED_X: f32 = 32.0;
const SCARED_Y: f32 = -16.0;

#[derive(Component)]
#[require(Sprite, Transform)]
struct Aggressive {
    moveable: bool,
    speed: f32,
}

impl Aggressive {
    fn acceleration(mut aggressive_query: Query<&mut Aggressive>, time: Res<Time>) -> Result {
        let mut aggressive = aggressive_query.single_mut()?;

        aggressive.speed += AGGRESSIVE_FORWARD_ACCELERATION * time.delta_secs();

        Ok(())
    }

    fn added(
        aggressive_query: Query<&mut Sprite, Added<Aggressive>>,
        control_assets: Res<ControlAssets>,
    ) {
        for mut sprite in aggressive_query {
            sprite.image = control_assets.aggressive.clone();
        }
    }

    fn check_release(
        mut aggressive_query: Query<&mut Aggressive>,
        input: Res<ButtonInput<KeyCode>>,
    ) -> Result {
        let mut aggressive = aggressive_query.single_mut()?;

        if input.just_released(KeyCode::KeyA) || input.just_released(KeyCode::ArrowLeft) {
            aggressive.moveable = true;
        }

        Ok(())
    }

    fn movement(
        mut aggressive_query: Query<(&mut Aggressive, &mut Transform)>,
        input: Res<ButtonInput<KeyCode>>,
        time: Res<Time>,
    ) -> Result {
        let (mut aggressive, mut transform) = aggressive_query.single_mut()?;

        if aggressive.moveable {
            let mut input_direction = Vec2::ZERO.clone();

            if input.pressed(KeyCode::KeyA) || input.pressed(KeyCode::ArrowLeft) {
                input_direction.x -= 1.0;
            }

            input_direction = input_direction.normalize_or_zero();

            if input_direction.length_squared() != 0.0 {
                let mut new_translation = transform.translation
                    + input_direction.extend(0.0) * AGGRESSIVE_BACK_SPEED * time.delta_secs();

                // Boundry checking
                new_translation.x = new_translation.x.min(MOVEABLE_HORIZONTAL_BOUNDRY);
                new_translation.x = new_translation.x.max(-MOVEABLE_HORIZONTAL_BOUNDRY);

                transform.translation = new_translation;

                // Cooldown
                aggressive.moveable = false;

                return Ok(());
            }
        }

        transform.translation += Vec3::X * aggressive.speed * time.delta_secs();

        Ok(())
    }

    fn new() -> impl Bundle {
        (
            Aggressive {
                moveable: true,
                speed: AGGRESSIVE_FORWARD_SPEED,
            },
            Transform::from_xyz(AGGRESSIVE_X, AGGRESSIVE_Y, 11.0),
        )
    }
}

#[derive(Component)]
#[require(Sprite, Transform)]
struct Scared;

impl Scared {
    fn added(scared_query: Query<&mut Sprite, Added<Scared>>, control_assets: Res<ControlAssets>) {
        for mut sprite in scared_query {
            sprite.image = control_assets.scared.clone();
        }
    }

    fn movement(mut scared_query: Query<&mut Transform, With<Scared>>, time: Res<Time>) -> Result {
        let mut transform = scared_query.single_mut()?;

        transform.translation += Vec3::X * SCARED_MOVEMENT_SPEED * time.delta_secs();

        Ok(())
    }

    fn new() -> impl Bundle {
        (Scared, Transform::from_xyz(SCARED_X, SCARED_Y, 10.0))
    }
}

#[derive(AssetCollection, Resource)]
pub struct ControlAssets {
    #[asset(path = "images/control_aggressive.png")]
    pub aggressive: Handle<Image>,
    #[asset(path = "images/control_background.png")]
    pub background: Handle<Image>,
    #[asset(path = "images/control_scared.png")]
    pub scared: Handle<Image>,
}

#[derive(Component)]
#[require(Sprite, Transform)]
struct Stage;

impl Stage {
    fn added(control_assets: Res<ControlAssets>, stage_query: Query<&mut Sprite, Added<Stage>>) {
        for mut sprite in stage_query {
            sprite.image = control_assets.background.clone();
        }
    }
}

fn check_collision(
    aggressive_query: Query<&Transform, With<Aggressive>>,
    mut commands: Commands,
    scared_query: Query<&Transform, With<Scared>>,
) -> Result {
    let aggressive_transform = aggressive_query.single()?;
    let scared_transform = scared_query.single()?;

    let aggressive_aabb = Aabb2d::new(
        aggressive_transform.translation.truncate(),
        AGGRESSIVE_BOUNDING_BOX,
    );
    let scared_aabb = Aabb2d::new(scared_transform.translation.truncate(), SCARED_BOUNDING_BOX);

    if aggressive_aabb.intersects(&scared_aabb) {
        commands.trigger(MinigameFinished(false));
    }

    Ok(())
}

pub(super) fn plugin(app: &mut App) {
    app.configure_loading_state(
        LoadingStateConfig::new(Screen::Loading).load_collection::<ControlAssets>(),
    );

    app.add_systems(
        Update,
        (
            (Aggressive::added, Scared::added, Stage::added).in_set(AppSystems::Update),
            (
                (Aggressive::check_release, Aggressive::movement).in_set(AppSystems::RecordInput),
                (Aggressive::acceleration, check_collision, Scared::movement)
                    .in_set(AppSystems::Update),
            )
                .run_if(in_minigame(MINIGAME_KEY))
                .in_set(PausableSystems),
        )
            .run_if(app_is_loaded),
    );
}

pub(super) fn spawn_minigame() -> impl Bundle {
    (Stage, children![Aggressive::new(), Scared::new()])
}
