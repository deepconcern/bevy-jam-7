use bevy::prelude::*;

use crate::{
    AppSystems,
    asset_tracking::LoadResource,
    game::{events::MinigameFinished, game_state::GameState},
};

#[derive(Asset, Clone, Reflect, Resource)]
#[reflect(Resource)]
pub struct DemoAssets {
    background: Handle<Image>,
}

impl FromWorld for DemoAssets {
    fn from_world(world: &mut World) -> Self {
        let mut texture_atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();

        let asset_server = world.resource::<AssetServer>();

        Self {
            background: asset_server.load("images/catch_background.png"),
        }
    }
}

fn end_game(mut commands: Commands, input: Res<ButtonInput<KeyCode>>) {
    if input.just_pressed(KeyCode::Enter) {
        commands.trigger(MinigameFinished(true));
    }
    if input.just_pressed(KeyCode::Backspace) {
        commands.trigger(MinigameFinished(false));
    }
}

pub fn spawn_minigame(demo_assets: Res<DemoAssets>) -> impl Bundle {
    (Sprite::from_image(demo_assets.background.clone()),)
}

pub fn plugin(app: &mut App) {
    app.load_resource::<DemoAssets>();

    app.add_systems(
        Update,
        (end_game.in_set(AppSystems::RecordInput)).run_if(in_state(GameState::Minigame)),
    );
}
