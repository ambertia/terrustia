use bevy::{
    color::palettes::tailwind::{AMBER_700, GREEN_700, STONE_500},
    prelude::*,
};

use crate::player::{PLAYER_HEIGHT, Player};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (build_ui, build_toolbar))
            .add_systems(Update, update_coordinates_ui);
    }
}

#[derive(Component)]
#[require(Text)]
struct UiCoordinateText;

fn update_coordinates_ui(
    mut text: Single<&mut Text, With<UiCoordinateText>>,
    player: Single<&Transform, With<Player>>,
) {
    text.0 = format!(
        "({0:.1}, {1:.1})",
        player.translation.x,
        player.translation.y - PLAYER_HEIGHT / 2.,
    );
}

fn build_ui(mut commands: Commands) {
    commands.spawn(UiCoordinateText);
}

/// Create the toolbar
fn build_toolbar(mut commands: Commands) {
    let toolbar_base = Node {
        margin: UiRect::all(Val::Px(5.)),
        column_gap: Val::Px(10.),
        justify_self: JustifySelf::End,
        ..default()
    };
    commands.spawn((
        toolbar_base,
        children![
            // This is a little ugly but it works just fine
            (
                ToolbarButtonBundle::default(),
                children![
                    ButtonTextLabel::new("1".to_owned()),
                    ButtonItemIcon::from_color(AMBER_700.into()),
                ],
            ),
            (
                ToolbarButtonBundle::default(),
                children![
                    ButtonTextLabel::new("2".to_owned()),
                    ButtonItemIcon::from_color(GREEN_700.into()),
                ],
            ),
            (
                ToolbarButtonBundle::default(),
                children![
                    ButtonTextLabel::new("3".to_owned()),
                    ButtonItemIcon::from_color(STONE_500.into()),
                ],
            ),
            (
                ToolbarButtonBundle::default(),
                children![
                    ButtonTextLabel::new("4".to_owned()),
                    ButtonItemIcon::from_color(AMBER_700.into()),
                ],
            ),
            (
                ToolbarButtonBundle::default(),
                children![
                    ButtonTextLabel::new("5".to_owned()),
                    ButtonItemIcon::from_color(GREEN_700.into()),
                ],
            ),
        ],
    ));
}

#[derive(Bundle)]
/// A bundle to simplify the creation of toolbar buttons with predefined properties
struct ToolbarButtonBundle {
    node: Node,
    border_radius: BorderRadius,
    border_color: BorderColor,
    background_color: BackgroundColor,
}

const TOOLBAR_SLOT_SIZE: f32 = 50.;
impl Default for ToolbarButtonBundle {
    fn default() -> Self {
        ToolbarButtonBundle {
            node: Node {
                height: Val::Px(TOOLBAR_SLOT_SIZE),
                width: Val::Px(TOOLBAR_SLOT_SIZE),
                border: UiRect::all(Val::Px(10.)),
                ..default()
            },
            border_radius: BorderRadius::all(Val::Px(5.)),
            border_color: BorderColor::from(Srgba::new(0.1, 0.1, 0.1, 1.)),
            background_color: BackgroundColor::from(Srgba::new(0.0, 0.0, 0.0, 0.4)),
        }
    }
}

#[derive(Bundle)]
struct ButtonTextLabel {
    node: Node,
    text: Text,
}

impl ButtonTextLabel {
    fn new(text: String) -> Self {
        ButtonTextLabel {
            node: Node::default(),
            text: Text(text),
        }
    }
}

#[derive(Bundle)]
struct ButtonItemIcon {
    node: Node,
    image: ImageNode,
}

impl ButtonItemIcon {
    fn from_color(color: Color) -> Self {
        ButtonItemIcon {
            node: Node::default(),
            image: ImageNode::solid_color(color),
        }
    }
}
