use bevy::prelude::*;
use bevy_asset_loader::{
    asset_collection::AssetCollection,
    loading_state::{
        LoadingStateAppExt,
        config::{ConfigureLoadingState, LoadingStateConfig},
    },
};

use crate::screens::Screen;

#[derive(AssetCollection, Resource)]
pub struct GameAssets {
    #[asset(path = "fonts/PressStart2P-Regular.ttf")]
    pub font: Handle<Font>,
    #[asset(path = "images/intermission.png")]
    pub interlude_background: Handle<Image>,
    #[asset(path = "images/lose_screen.png")]
    pub lose_screen: Handle<Image>,
    #[asset(path = "images/thermometer.png")]
    pub thermometer: Handle<Image>,
    #[asset(texture_atlas_layout(tile_size_x = 64, tile_size_y = 32, columns = 2, rows = 2))]
    pub thermometer_layout: Handle<TextureAtlasLayout>,
    #[asset(path = "images/thermometer_numbers.png")]
    pub thermometer_numbers: Handle<Image>,
    #[asset(texture_atlas_layout(tile_size_x = 8, tile_size_y = 16, columns = 10, rows = 1))]
    pub thermometer_numbers_layout: Handle<TextureAtlasLayout>,
    #[asset(path = "images/ui.png")]
    pub ui_background: Handle<Image>,
    #[asset(path = "images/win_screen.png")]
    pub win_screen: Handle<Image>,
}

pub(super) fn plugin(app: &mut App) {
    app.configure_loading_state(
        LoadingStateConfig::new(Screen::Loading).load_collection::<GameAssets>(),
    );
}
