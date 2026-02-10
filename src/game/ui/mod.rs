mod thermometer;

use bevy::prelude::*;

pub use thermometer::{GAME_OVER_FEVER, NO_FEVER, Thermometer, thermometer};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(thermometer::plugin);
}
