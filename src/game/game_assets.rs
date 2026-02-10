use bevy::prelude::*;

use crate::asset_tracking::LoadResource;

const THERMOMETER_HEIGHT: f32 = 32.0;
const THERMOMETER_WIDTH: f32 = 64.0;
const THERMOMETER_NUMBER_HEIGHT: f32 = 16.0;
const THERMOMETER_NUMBER_WIDTH: f32 = 8.0;

#[derive(Asset, Clone, Reflect, Resource)]
#[reflect(Resource)]
pub struct GameAssets {
    pub font: Handle<Font>,
    pub interlude_background: Handle<Image>,
    pub lose_screen: Handle<Image>,
    pub thermometer: Handle<Image>,
    pub thermometer_layout: Handle<TextureAtlasLayout>,
    pub thermometer_numbers: Handle<Image>,
    pub thermometer_numbers_layout: Handle<TextureAtlasLayout>,
    pub ui_background: Handle<Image>,
    pub win_screen: Handle<Image>,
}

impl FromWorld for GameAssets {
    fn from_world(world: &mut World) -> Self {
        let mut texture_atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();

        let thermometer_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
            UVec2::new(THERMOMETER_WIDTH as u32, THERMOMETER_HEIGHT as u32),
            2,
            2,
            None,
            None,
        ));

        let thermometer_numbers_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
            UVec2::new(
                THERMOMETER_NUMBER_WIDTH as u32,
                THERMOMETER_NUMBER_HEIGHT as u32,
            ),
            10,
            1,
            None,
            None,
        ));

        let asset_server = world.resource::<AssetServer>();

        Self {
            font: asset_server.load("fonts/PressStart2P-Regular.ttf"),
            interlude_background: asset_server.load("images/intermission.png"),
            lose_screen: asset_server.load("images/lose_screen.png"),
            thermometer: asset_server.load("images/thermometer.png"),
            thermometer_layout,
            thermometer_numbers: asset_server.load("images/thermometer_numbers.png"),
            thermometer_numbers_layout,
            ui_background: asset_server.load("images/ui.png"),
            win_screen: asset_server.load("images/win_screen.png"),
        }
    }
}

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<GameAssets>();
}
