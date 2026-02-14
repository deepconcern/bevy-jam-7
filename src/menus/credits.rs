//! The credits menu.

use bevy::{ecs::spawn::SpawnIter, input::common_conditions::input_just_pressed, prelude::*};
use bevy_asset_loader::prelude::*;

use crate::{audio::music, menus::Menu, screens::Screen, theme::prelude::*};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Credits), spawn_credits_menu);
    app.add_systems(
        Update,
        go_back.run_if(in_state(Menu::Credits).and(input_just_pressed(KeyCode::Escape))),
    );

    app.configure_loading_state(
        LoadingStateConfig::new(Screen::Loading).load_collection::<CreditsAssets>(),
    );
    app.add_systems(OnEnter(Menu::Credits), start_credits_music);
}

fn spawn_credits_menu(mut commands: Commands) {
    commands.spawn((
        widget::ui_root("Credits Menu", false),
        GlobalZIndex(2),
        DespawnOnExit(Menu::Credits),
        children![
            widget::header("Created by"),
            created_by(),
            widget::header("Assets"),
            assets(),
            widget::button("Back", go_back_on_click),
        ],
    ));
}

fn created_by() -> impl Bundle {
    grid(vec![["Wyatt Barnes", "Design/Gameplay/Art/Music"]])
}

fn assets() -> impl Bundle {
    grid(vec![
        [
            "\"Press Start 2P\" Font",
            "cody@zone38.net (SIL OPEN FONT LICENSE V1.1)",
        ],
        [
            "Bevy logo",
            "All rights reserved by the Bevy Foundation, permission granted for splash screen use when unmodified",
        ],
    ])
}

fn grid(content: Vec<[&'static str; 2]>) -> impl Bundle {
    (
        Name::new("Grid"),
        Node {
            display: Display::Grid,
            row_gap: px(10),
            column_gap: px(30),
            grid_template_columns: RepeatedGridTrack::px(2, 400.0),
            ..default()
        },
        Children::spawn(SpawnIter(content.into_iter().flatten().enumerate().map(
            |(i, text)| {
                (
                    widget::label(text),
                    Node {
                        justify_self: if i.is_multiple_of(2) {
                            JustifySelf::End
                        } else {
                            JustifySelf::Start
                        },
                        ..default()
                    },
                )
            },
        ))),
    )
}

fn go_back_on_click(_: On<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Main);
}

fn go_back(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Main);
}

#[derive(AssetCollection, Resource)]
struct CreditsAssets {
    #[asset(path = "audio/music/Monkeys Spinning Monkeys.ogg")]
    music: Handle<AudioSource>,
}

fn start_credits_music(mut commands: Commands, credits_music: Res<CreditsAssets>) {
    commands.spawn((
        Name::new("Credits Music"),
        DespawnOnExit(Menu::Credits),
        music(credits_music.music.clone()),
    ));
}
