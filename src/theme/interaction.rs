use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

use crate::{audio::sound_effect, screens::Screen};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(apply_interaction_palette_on_click);
    app.add_observer(apply_interaction_palette_on_over);
    app.add_observer(apply_interaction_palette_on_out);
    app.add_observer(apply_interaction_palette_on_release);

    app.configure_loading_state(
        LoadingStateConfig::new(Screen::Loading).load_collection::<InteractionAssets>(),
    );
    app.add_observer(play_sound_effect_on_click);
    app.add_observer(play_sound_effect_on_over);
}

/// Palette for widget interactions. Add this to an entity that supports
/// [`Interaction`]s, such as a button, to change its [`BackgroundColor`] based
/// on the current interaction state.
#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct InteractionPalette {
    pub none: Color,
    pub hovered: Color,
    pub pressed: Color,
}

fn apply_interaction_palette_on_click(
    click: On<Pointer<Click>>,
    mut palette_query: Query<(&InteractionPalette, &mut BackgroundColor)>,
) {
    let Ok((palette, mut bg)) = palette_query.get_mut(click.event_target()) else {
        return;
    };

    *bg = palette.pressed.into();
}

fn apply_interaction_palette_on_release(
    click: On<Pointer<Release>>,
    mut palette_query: Query<(&InteractionPalette, &mut BackgroundColor)>,
) {
    let Ok((palette, mut bg)) = palette_query.get_mut(click.event_target()) else {
        return;
    };

    *bg = palette.hovered.into();
}

fn apply_interaction_palette_on_over(
    over: On<Pointer<Over>>,
    mut palette_query: Query<(&InteractionPalette, &mut BackgroundColor)>,
) {
    let Ok((palette, mut bg)) = palette_query.get_mut(over.event_target()) else {
        return;
    };

    *bg = palette.hovered.into();
}

fn apply_interaction_palette_on_out(
    out: On<Pointer<Out>>,
    mut palette_query: Query<(&InteractionPalette, &mut BackgroundColor)>,
) {
    let Ok((palette, mut bg)) = palette_query.get_mut(out.event_target()) else {
        return;
    };

    *bg = palette.none.into();
}

#[derive(AssetCollection, Resource)]
struct InteractionAssets {
    #[asset(path = "audio/sound_effects/button_hover.ogg")]
    hover: Handle<AudioSource>,
    #[asset(path = "audio/sound_effects/button_click.ogg")]
    click: Handle<AudioSource>,
}

fn play_sound_effect_on_click(
    _: On<Pointer<Click>>,
    interaction_assets: If<Res<InteractionAssets>>,
    mut commands: Commands,
) {
    commands.spawn(sound_effect(interaction_assets.click.clone()));
}

fn play_sound_effect_on_over(
    _: On<Pointer<Over>>,
    interaction_assets: If<Res<InteractionAssets>>,
    mut commands: Commands,
) {
    commands.spawn(sound_effect(interaction_assets.hover.clone()));
}
