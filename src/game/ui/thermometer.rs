use bevy::prelude::*;

use crate::game::game_assets::GameAssets;

pub const GAME_OVER_FEVER: f32 = 41.0;
const HIGH_FEVER: f32 = 40.0;
const STARTING_FEVER: f32 = 39.0;
const LOW_FEVER: f32 = 38.0;
pub const NO_FEVER: f32 = 37.0;

#[derive(Component)]
pub struct Thermometer {
    pub reading: f32,
}

pub enum Digit {
    Tens,
    Ones,
    Tenths,
}

#[derive(Component)]
pub struct ThermometerNumber(Digit);

fn render_thermometer(
    mut thermometer_query: Query<
        (&Children, &mut Sprite, &Thermometer),
        Without<ThermometerNumber>,
    >,
    mut thermometer_number_query: Query<(&mut Sprite, &ThermometerNumber), Without<Thermometer>>,
) {
    let Ok((children, mut sprite, thermometer)) = thermometer_query.single_mut() else {
        return;
    };

    let Some(texture_atlas) = sprite.texture_atlas.as_mut() else {
        return;
    };

    if thermometer.reading >= HIGH_FEVER {
        texture_atlas.index = 2;
    } else if thermometer.reading <= LOW_FEVER {
        texture_atlas.index = 0;
    } else {
        texture_atlas.index = 1;
    }

    for child in children {
        let Ok((mut sprite, thermometer_number)) = thermometer_number_query.get_mut(*child) else {
            continue;
        };

        let Some(texture_atlas) = sprite.texture_atlas.as_mut() else {
            continue;
        };

        let digit = (match thermometer_number.0 {
            Digit::Ones => thermometer.reading,
            Digit::Tens => (thermometer.reading / 10.0).floor(),
            Digit::Tenths => (thermometer.reading * 10.0).floor(),
        }) as usize
            % 10;

        texture_atlas.index = digit;
    }
}

pub fn thermometer(game_assets: Res<GameAssets>) -> impl Bundle {
    (
        Sprite::from_atlas_image(
            game_assets.thermometer.clone(),
            TextureAtlas {
                index: 0,
                layout: game_assets.thermometer_layout.clone(),
            },
        ),
        Thermometer {
            reading: STARTING_FEVER,
        },
        Transform::from_xyz(0.0, -48.0, 1.0).with_scale(Vec3::splat(2.0)),
        children![
            (
                Sprite::from_atlas_image(
                    game_assets.thermometer_numbers.clone(),
                    TextureAtlas {
                        index: 0,
                        layout: game_assets.thermometer_numbers_layout.clone(),
                    }
                ),
                ThermometerNumber(Digit::Tens),
                Transform::from_xyz(-9.0, 0.0, 1.0)
            ),
            (
                Sprite::from_atlas_image(
                    game_assets.thermometer_numbers.clone(),
                    TextureAtlas {
                        index: 0,
                        layout: game_assets.thermometer_numbers_layout.clone(),
                    }
                ),
                ThermometerNumber(Digit::Ones),
                Transform::from_xyz(-2.0, 0.0, 1.0)
            ),
            (
                Sprite::from_atlas_image(
                    game_assets.thermometer_numbers.clone(),
                    TextureAtlas {
                        index: 0,
                        layout: game_assets.thermometer_numbers_layout.clone(),
                    }
                ),
                ThermometerNumber(Digit::Tenths),
                Transform::from_xyz(6.0, 0.0, 1.0)
            )
        ],
    )
}

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, render_thermometer);
}
