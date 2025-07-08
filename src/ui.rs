use bevy::{
    color::palettes::tailwind::{AMBER_700, GREEN_700, STONE_500},
    ecs::{component::HookContext, world::DeferredWorld},
    prelude::*,
};

use crate::{
    inventory::ItemStack,
    player::{PLAYER_HEIGHT, Player},
};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Toolbar>()
            .add_event::<ToolbarSlotUpdate>()
            .add_systems(Startup, (build_ui, build_toolbar))
            .add_systems(
                Update,
                (update_coordinates_ui, keyboard_toolbar, update_toolbar_slot),
            );
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
pub const TOOLBAR_BUTTONS: usize = 5;
fn build_toolbar(mut commands: Commands, mut toolbar: ResMut<Toolbar>) {
    let toolbar_base = Node {
        margin: UiRect::all(Val::Px(5.)),
        column_gap: Val::Px(10.),
        justify_self: JustifySelf::End,
        ..default()
    };

    // Vecs to use to accumulate the toolbar elements
    let mut buttons: Vec<Entity> = Vec::new();
    let mut icons: Vec<Entity> = Vec::new();
    let mut texts: Vec<Entity> = Vec::new();

    commands.spawn(toolbar_base).with_children(|p| {
        for _ in 0..TOOLBAR_BUTTONS {
            buttons.push(
                p.spawn(ToolbarButtonBundle::default())
                    .with_children(|p| {
                        icons.push(p.spawn(ButtonItemIcon::default()).id());
                        texts.push(p.spawn(ButtonTextLabel::default()).id());
                    })
                    .id(),
            );
        }
    });

    // Move the Vecs to the Resource things
    toolbar.buttons = buttons;
    toolbar.icons = icons;
    toolbar.text = texts;
}

#[derive(Resource, Default)]
pub struct Toolbar {
    pub buttons: Vec<Entity>,
    icons: Vec<Entity>,
    text: Vec<Entity>,
    pub selected: usize,
}

#[derive(Event)]
pub struct ToolbarSlotUpdate {
    stack: Option<ItemStack>,
    slot: usize,
}

fn update_toolbar_slot(
    mut events: EventReader<ToolbarSlotUpdate>,
    toolbar: Res<Toolbar>,
    mut commands: Commands,
) {
    for e in events.read() {
        // Try to get the icon entity and text entity from the toolbar. If we can't (e.g. somehow
        // e.slot is higher than the actual number of toolbar slots), then skip this event.
        let Some(icon_entity) = toolbar.icons.get(e.slot) else {
            return;
        };
        let Some(text_entity) = toolbar.text.get(e.slot) else {
            return;
        };

        // Get a color based on the item id
        // In the future this will be more complicated
        let image_node = match e.stack {
            Some(s) => ImageNode::solid_color(Color::from(match s.item_id {
                1 => AMBER_700,
                2 => GREEN_700,
                _ => STONE_500,
            })),
            None => ImageNode::default(),
        };

        // Get text from the count
        let text = Text(match e.stack {
            Some(s) => format!("{}", s.count),
            None => "".to_owned(),
        });

        // Apply the new properties to the respective entities
        commands.entity(icon_entity.to_owned()).insert(image_node);
        commands.entity(text_entity.to_owned()).insert(text);
    }
}

/// Marker component for toolbar buttons
#[derive(Component)]
struct ToolbarButton;

#[derive(Bundle)]
/// A bundle to simplify the creation of toolbar buttons with predefined properties
struct ToolbarButtonBundle {
    marker: ToolbarButton,
    node: Node,
    border_radius: BorderRadius,
    border_color: BorderColor,
    background_color: BackgroundColor,
}

const TOOLBAR_SLOT_SIZE: f32 = 50.;
impl Default for ToolbarButtonBundle {
    fn default() -> Self {
        ToolbarButtonBundle {
            marker: ToolbarButton,
            node: Node {
                height: Val::Px(TOOLBAR_SLOT_SIZE),
                width: Val::Px(TOOLBAR_SLOT_SIZE),
                border: UiRect::all(Val::Px(2.)),
                display: Display::Grid,
                ..default()
            },
            border_radius: BorderRadius::all(Val::Px(5.)),
            border_color: BorderColor::from(Srgba::new(0.1, 0.1, 0.1, 0.6)),
            background_color: BackgroundColor::from(Srgba::new(0.0, 0.0, 0.0, 0.4)),
        }
    }
}

/// Marker component for the text on the toolbar buttons
#[derive(Component)]
struct ToolbarButtonText;

#[derive(Bundle)]
/// A bundle to ease the spawning of standardized Text (item count) labels for the toolbar buttons
struct ButtonTextLabel {
    marker: ToolbarButtonText,
    node: Node,
    text: Text,
    text_font: TextFont,
    z_index: ZIndex,
}

impl ButtonTextLabel {
    fn _new(text: String) -> Self {
        let mut this_label = ButtonTextLabel::default();
        this_label.text = Text(text);
        this_label
    }
}

impl Default for ButtonTextLabel {
    fn default() -> Self {
        ButtonTextLabel {
            marker: ToolbarButtonText,
            node: Node {
                grid_row: GridPlacement::start_end(1, 1),
                grid_column: GridPlacement::start_end(1, 1),
                justify_self: JustifySelf::End,
                align_self: AlignSelf::End,
                ..default()
            },
            text: Text::default(),
            text_font: TextFont::default()
                .with_line_height(bevy::text::LineHeight::RelativeToFont(1.)),
            z_index: ZIndex(1),
        }
    }
}

#[derive(Component)]
struct ToolbarIcon;

#[derive(Bundle)]
/// A bundle to ease the spawning of standardized ImageNodes for the toolbar buttons
struct ButtonItemIcon {
    marker: ToolbarIcon,
    node: Node,
    image: ImageNode,
}

impl ButtonItemIcon {
    fn _from_color(color: Color) -> Self {
        let mut this_icon = ButtonItemIcon::default();
        this_icon.image = ImageNode::solid_color(color);
        this_icon
    }
}

impl Default for ButtonItemIcon {
    fn default() -> Self {
        ButtonItemIcon {
            marker: ToolbarIcon,
            node: Node {
                height: Val::Percent(70.),
                width: Val::Percent(70.),
                justify_self: JustifySelf::Center,
                align_self: AlignSelf::Center,
                grid_row: GridPlacement::start_end(1, 1),
                grid_column: GridPlacement::start_end(1, 1),
                ..default()
            },
            image: ImageNode::default(),
        }
    }
}

#[derive(Event)]
/// Update a toolbar button's visual appearance to match a given ItemStack
pub struct UpdateToolbarButton(pub Option<ItemStack>);

/// Change the appearance of a toolbar button when the contents of the slot it represents change
fn update_toolbar_button(
    trigger: Trigger<UpdateToolbarButton>,
    children: Query<&Children, With<ToolbarButton>>,
    texts: Query<&Text>,
    image_nodes: Query<&ImageNode>,
    mut commands: Commands,
) {
    // Get the inventory slot data from the trigger. If the slot is empty, we can update the fields
    // immediately and return
    let Some(stack) = trigger.0 else {
        commands
            .entity(trigger.target())
            .insert((Text::new(""), ImageNode::default()));
        return;
    };

    // Iterate over the Children of the ToolbarButton
    for e in children.get(trigger.target()).unwrap().iter() {
        // Update the button text when we find it
        if let Ok(_) = texts.get(e) {
            commands.entity(e).insert(Text::new(match stack.count {
                0 => "".to_owned(), // There shouldn't technically be stacks with 0 count but...
                _ => format!("{}", stack.count),
            }));
        }

        // Update the icon when we find it
        if let Ok(_) = image_nodes.get(e) {
            // Get color based on which block it is
            commands.entity(e).insert(ImageNode::solid_color(
                match stack.item_id {
                    1 => AMBER_700,
                    2 => GREEN_700,
                    _ => STONE_500,
                }
                .into(),
            ));
        }
    }
}

fn keyboard_toolbar(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut toolbar: ResMut<Toolbar>,
    mut commands: Commands,
) {
    // TODO: This has bad code smell but it's a straightforward structure and the docs say
    // just_pressed() runs in constant time
    if keyboard.just_pressed(KeyCode::Digit1) {
        commands
            .entity(toolbar.buttons.get(toolbar.selected).unwrap().to_owned())
            .insert(BorderColor::from(Srgba::new(0., 0., 0., 0.6)));
        toolbar.selected = 0;
        commands
            .entity(toolbar.buttons.get(toolbar.selected).unwrap().to_owned())
            .insert(BorderColor::from(Srgba::new(0., 0., 0., 1.)));
    } else if keyboard.just_pressed(KeyCode::Digit2) {
        commands
            .entity(toolbar.buttons.get(toolbar.selected).unwrap().to_owned())
            .insert(BorderColor::from(Srgba::new(0., 0., 0., 0.6)));
        toolbar.selected = 1;
        commands
            .entity(toolbar.buttons.get(toolbar.selected).unwrap().to_owned())
            .insert(BorderColor::from(Srgba::new(0., 0., 0., 1.)));
    } else if keyboard.just_pressed(KeyCode::Digit3) {
        commands
            .entity(toolbar.buttons.get(toolbar.selected).unwrap().to_owned())
            .insert(BorderColor::from(Srgba::new(0., 0., 0., 0.6)));
        toolbar.selected = 2;
        commands
            .entity(toolbar.buttons.get(toolbar.selected).unwrap().to_owned())
            .insert(BorderColor::from(Srgba::new(0., 0., 0., 1.)));
    } else if keyboard.just_pressed(KeyCode::Digit4) {
        commands
            .entity(toolbar.buttons.get(toolbar.selected).unwrap().to_owned())
            .insert(BorderColor::from(Srgba::new(0., 0., 0., 0.6)));
        toolbar.selected = 3;
        commands
            .entity(toolbar.buttons.get(toolbar.selected).unwrap().to_owned())
            .insert(BorderColor::from(Srgba::new(0., 0., 0., 1.)));
    } else if keyboard.just_pressed(KeyCode::Digit5) {
        commands
            .entity(toolbar.buttons.get(toolbar.selected).unwrap().to_owned())
            .insert(BorderColor::from(Srgba::new(0., 0., 0., 0.6)));
        toolbar.selected = 4;
        commands
            .entity(toolbar.buttons.get(toolbar.selected).unwrap().to_owned())
            .insert(BorderColor::from(Srgba::new(0., 0., 0., 1.)));
    }
}
