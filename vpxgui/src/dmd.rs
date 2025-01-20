use bevy::color::palettes::css::{GHOST_WHITE, GOLDENROD};
use bevy::color::Color;
use bevy::math::Vec3;
use bevy::prelude::{
    default, BackgroundColor, BorderRadius, BoxShadow, Bundle, Commands, Component, JustifyText,
    LineBreak, Node, Query, Res, Text, TextColor, TextFont, TextLayout, Transform, UiRect, Val,
    Visibility, Window, With,
};
use bevy::window::PrimaryWindow;
use bevy_asset::AssetServer;

#[derive(Component)]
pub struct Dmd;

#[derive(Bundle)]
struct DmdBundle {
    node: Node,
    //sprite: Sprite,
    transform: Transform,
    boxshadow: BoxShadow,
    backgroundcolor: BackgroundColor,
    borderradius: BorderRadius,
    // translate: Translate,
    //global_transform: GlobalTransform,
    //    visibility: Visibility,
    //    wheel: Wheel,
    //inherited_visibility: InheritedVisibility,
    visibility: Visibility,
    text: Text,
    text_font: TextFont,
    text_color: TextColor,
    text_layout: TextLayout,
    // table_text: TableText,
    //text_bundle: Node,
    dmd: Dmd,
}

pub(crate) fn create_dmd(
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let window = window_query.single();
    let window_width = window.width();
    let window_height = window.height();
    commands.spawn(DmdBundle {
        node: Node {
            width: Val::Px(512.),
            height: Val::Px(128.),
            //left: Val::Px(10.),
            left: Val::Px(window_width / 6.),
            top: Val::Px(window_height / 2.),
            border: UiRect::all(Val::Px(2.)),

            ..Default::default()
        },
        visibility: Visibility::Hidden,
        transform: Transform {
            translation: Vec3::new(
                window_width - (window_width * 0.60) - 225.,
                (window_height * 0.25) + 60.,
                0.,
            ),
            ..default()
        },

        boxshadow: BoxShadow {
            color: GOLDENROD.into(),
            x_offset: Val::Px(0.),
            y_offset: Val::Px(0.),
            spread_radius: Val::Px(20.),
            blur_radius: Val::Px(2.),
        },
        backgroundcolor: BackgroundColor(Color::srgba(0.5, 0.5, 0.5, 1.0)),
        borderradius: BorderRadius::new(
            // top left
            Val::Px(40.),
            // top right
            Val::Px(40.),
            // bottom right
            Val::Px(40.),
            // bottom left
            Val::Px(40.),
        ),
        dmd: Dmd,
        text_layout: TextLayout {
            justify: JustifyText::Center,
            linebreak: LineBreak::WordBoundary,
        },
        //text_layout: TextLayout::new_with_justify(JustifyText::Center).with_no_wrap(),
        text: Text::new("Keys       q: quit\n1: open up table description dialog\nleft-shift: scroll backward\nright-shift: scroll forward\nenter: start selected game"),
        text_font: TextFont {
            // This font is loaded and will be used instead of the default font.
            font_size: 20.0,
            ..default()
        },
        text_color: TextColor::from(GHOST_WHITE),
        // Set the justification of the Text
        //.with_text_justify(JustifyText::Center)
        // Set the style of the TextBundle itself.
    });
}
