//! Helper functions for creating common widgets.

use std::borrow::Cow;

use bevy::{
    ecs::{spawn::SpawnWith, system::IntoObserverSystem},
    prelude::*,
};
use bevy_asset_loader::prelude::*;

use crate::{
    AppSystems, app_is_loaded,
    game::game_assets::GameAssets,
    screens::Screen,
    theme::{interaction::InteractionPalette, palette::*},
};

#[derive(Component)]
struct UiRoot(bool);

#[derive(Component)]
struct Widget;

/// A root UI node that fills the window and centers its content.
pub fn ui_root(name: impl Into<Cow<'static, str>>, hide_background: bool) -> impl Bundle {
    (
        Widget,
        UiRoot(hide_background),
        Name::new(name),
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            row_gap: px(20),
            ..default()
        },
        // Don't block picking events for other UI roots.
        Pickable::IGNORE,
    )
}

/// A simple header label. Bigger than [`label`].
pub fn header(text: impl Into<String>) -> impl Bundle {
    (
        Widget,
        Name::new("Header"),
        Text(text.into()),
        TextFont::from_font_size(40.0),
        TextColor(HEADER_TEXT),
    )
}

/// A simple text label.
pub fn label(text: impl Into<String>) -> impl Bundle {
    (
        Widget,
        Name::new("Label"),
        Text(text.into()),
        TextFont::from_font_size(18.0),
        TextColor(LABEL_TEXT),
    )
}

/// A large rounded button with text and an action defined as an [`Observer`].
pub fn button<E, B, M, I>(text: impl Into<String>, action: I) -> impl Bundle
where
    E: EntityEvent,
    B: Bundle,
    I: IntoObserverSystem<E, B, M>,
{
    button_base(
        text,
        action,
        Node {
            width: px(200),
            height: px(32),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
    )
}

/// A small square button with text and an action defined as an [`Observer`].
pub fn button_small<E, B, M, I>(text: impl Into<String>, action: I) -> impl Bundle
where
    E: EntityEvent,
    B: Bundle,
    I: IntoObserverSystem<E, B, M>,
{
    button_base(
        text,
        action,
        Node {
            width: px(30),
            height: px(30),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
    )
}

/// A simple button with text and an action defined as an [`Observer`]. The button's layout is provided by `button_bundle`.
fn button_base<E, B, M, I>(
    text: impl Into<String>,
    action: I,
    button_bundle: impl Bundle,
) -> impl Bundle
where
    E: EntityEvent,
    B: Bundle,
    I: IntoObserverSystem<E, B, M>,
{
    let text = text.into();
    let action = IntoObserverSystem::into_system(action);
    (
        Name::new("Button"),
        Node::default(),
        Children::spawn(SpawnWith(|parent: &mut ChildSpawner| {
            parent
                .spawn((
                    Name::new("Button Inner"),
                    Button,
                    BackgroundColor(BUTTON_BACKGROUND),
                    InteractionPalette {
                        none: BUTTON_BACKGROUND,
                        hovered: BUTTON_HOVERED_BACKGROUND,
                        pressed: BUTTON_PRESSED_BACKGROUND,
                    },
                    children![(
                        Widget,
                        Name::new("Button Text"),
                        Text(text),
                        TextFont::from_font_size(16.0),
                        TextColor(BUTTON_TEXT),
                        // Don't bubble picking events from the text up to the button.
                        Pickable::IGNORE,
                    )],
                ))
                .insert(button_bundle)
                .observe(action);
        })),
    )
}

fn widget_font_added(
    game_assets: Res<GameAssets>,
    text_font_query: Query<&mut TextFont, Added<Widget>>,
) {
    for mut text_font in text_font_query {
        text_font.font = game_assets.font.clone();
    }
}

fn ui_root_added(
    mut commands: Commands,
    ui_root_query: Query<(Entity, &UiRoot), Added<UiRoot>>,
    widget_assets: Res<WidgetAssets>,
) {
    for (ui_root_entity, ui_root) in ui_root_query {
        if ui_root.0 {
            continue;
        }

        commands
            .entity(ui_root_entity)
            .insert(ImageNode::new(widget_assets.main_bg.clone()));
    }
}

#[derive(AssetCollection, Resource)]
struct WidgetAssets {
    #[asset(path = "images/title_screen.png")]
    main_bg: Handle<Image>,
}

pub(super) fn plugin(app: &mut App) {
    app.configure_loading_state(
        LoadingStateConfig::new(Screen::Loading).load_collection::<WidgetAssets>(),
    );

    app.add_systems(
        Update,
        (ui_root_added, widget_font_added)
            .in_set(AppSystems::Update)
            .run_if(app_is_loaded),
    );
}
