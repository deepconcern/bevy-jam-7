use bevy::prelude::*;

use crate::{
    AppSystems,
    game::{events::MinigameFinished, game_state::GameState},
};

pub const MINIGAME_KEY: &'static str = "demo";

#[derive(Component)]
#[require(Sprite, Transform)]
struct Stage;

fn end_game(mut commands: Commands, input: Res<ButtonInput<KeyCode>>) {
    if input.just_pressed(KeyCode::Enter) {
        commands.trigger(MinigameFinished(true));
    }
    if input.just_pressed(KeyCode::Backspace) {
        commands.trigger(MinigameFinished(false));
    }
}

fn stage_added(
    observe_assets: Res<super::observe::ObserveAssets>,
    mut stage_query: Query<&mut Sprite, Added<Stage>>,
) {
    let Ok(mut sprite) = stage_query.single_mut() else {
        return;
    };

    sprite.image = observe_assets.background.clone();
}

pub fn spawn_minigame() -> impl Bundle {
    Stage
}

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            stage_added.in_set(AppSystems::Update),
            (end_game.in_set(AppSystems::RecordInput),)
                .run_if(in_state(GameState::Minigame(MINIGAME_KEY.to_string()))),
        ),
    );
}
