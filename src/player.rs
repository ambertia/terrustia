use avian2d::{math::Vector, prelude::*};
use bevy::{
    color::palettes::{css::WHITE, tailwind::GRAY_950},
    prelude::*,
};

pub struct CharacterControllerPlugin;

impl Plugin for CharacterControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                (update_grounded, keyboard_input).chain(),
                handle_item_pickups,
            ),
        )
        .add_systems(Startup, (build_toolbar, spawn_player))
        .init_resource::<PlayerInventory>()
        .add_event::<ItemPickedUp>();
    }
}

/// Marker component to add player controller logic to an entity
#[derive(Component)]
#[require(RigidBody)]
pub struct Player;

/// Mark whether or not the player is on the ground for jump logic. Change storage settings since
/// this is a marker that should be added and removed quickly
#[derive(Component)]
#[component(storage = "SparseSet")]
struct Grounded;

/// Tolerance in radians defining allowable "slope" that is still considered a grounding collision.
/// Since the world is made up of square tiles, it should be fine to have a small but nonzero
/// tolerance.
const HIT_TOLERANCE_RADIANS: f32 = 0.1;
/// Update the Grounded state of the player using its shape caster
fn update_grounded(player: Single<(Entity, &ShapeHits), With<Player>>, mut commands: Commands) {
    let (player_entity, caster_hits) = player.into_inner();

    // Iterate over every collision occuring with the Player. If there is a collision with normal
    // facing upward, the player is grounded
    if caster_hits
        .iter()
        .any(|hit| -hit.normal2.angle_to(Vector::Y).abs() < HIT_TOLERANCE_RADIANS)
    {
        commands.entity(player_entity).insert(Grounded);
    } else {
        commands.entity(player_entity).remove::<Grounded>();
    }
}

const HORIZONTAL_VELOCITY_MAX: f32 = 20.;
const HORIZONTAL_ACCELERATION: f32 = 10.;
const JUMP_VEL: f32 = 20.;
/// Check for input every frame
fn keyboard_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    player: Single<(&mut LinearVelocity, Has<Grounded>), With<Player>>,
) {
    let (mut player_vel, player_grounded) = player.into_inner();

    // Get horizontal direction from A/D
    let left = keyboard.pressed(KeyCode::KeyA) as i8;
    let right = keyboard.pressed(KeyCode::KeyD) as i8;
    // Accelerate horizontal velocity
    player_vel.x += HORIZONTAL_ACCELERATION * f32::from(right - left) * time.delta_secs();

    // If W / Space is pressed and the player is grounded, set their velocity to a fixed value
    if player_grounded {
        if keyboard.any_pressed([KeyCode::KeyW, KeyCode::Space]) {
            player_vel.y = JUMP_VEL;
        }
    }
}

const PLAYER_WIDTH: f32 = 2.;
const PLAYER_HEIGHT: f32 = 3.;

fn spawn_player(mut commands: Commands) {
    commands.spawn((
        Player,
        RigidBody::Dynamic,
        Collider::rectangle(PLAYER_WIDTH - 0.1, PLAYER_HEIGHT - 0.1),
        Sprite {
            color: Color::from(WHITE),
            custom_size: Some(Vec2::new(PLAYER_WIDTH, PLAYER_HEIGHT)),
            ..default()
        },
        Transform::from_xyz(0., 30., 1.),
        // A ShapeCaster to help detect if the player is touching the ground.
        ShapeCaster::new(
            Collider::rectangle(PLAYER_WIDTH * 0.99, PLAYER_HEIGHT * 0.99),
            Vector::ZERO,
            0.,
            Dir2::NEG_Y,
        )
        .with_max_distance(0.1),
        LockedAxes::ROTATION_LOCKED,
        Friction::new(0.1).with_combine_rule(CoefficientCombine::Min),
        CollisionMargin(0.05),
        LinearDamping(0.1),
    ));
}

#[derive(Resource, Default)]
/// Resource to contain the player's inventory information
// This only needs to hold an array of block id's for now because the only interactable blocks are
// the three types of foreground blocks, which are all stackable. This will change in the future
// and require more complex inventory management.
// Option should default to None which is perfect.
struct PlayerInventory([Option<ItemStack>; 5]);

struct ItemStack {
    count: usize,
    item_id: usize,
}

#[derive(Event)]
pub struct ItemPickedUp(pub usize);

/// Process all pending ItemPickedUp events and modify the player's inventory accordingly
fn handle_item_pickups(
    mut events: EventReader<ItemPickedUp>,
    mut inventory: ResMut<PlayerInventory>,
) {
    for event in events.read() {
        let mut first_empty_slot: Option<usize> = None;
        // Iterate over all inventory slots
        for i in 0..(inventory.0.len()) {
            match &inventory.0[i] {
                // If the slot has a stack with matching item_id, put the item in this stack
                Some(s) if s.item_id == event.0 => {
                    inventory.0[i] = Some(ItemStack {
                        item_id: s.item_id,
                        count: s.count + 1,
                    });
                    return;
                }
                // Track the first empty inventory slot we find, if any
                None if first_empty_slot.is_none() => first_empty_slot = Some(i),
                _ => {}
            }
        }
        // If no such stack exists, put the item in the first empty slot
        if let Some(i) = first_empty_slot {
            inventory.0[i] = Some(ItemStack {
                item_id: event.0,
                count: 1,
            });
        }
    }
}

const TOOLBAR_SLOT_SIZE: f32 = 50.;
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
            ToolbarButtonBundle::default(),
            ToolbarButtonBundle::default(),
            ToolbarButtonBundle::default(),
            ToolbarButtonBundle::default(),
            ToolbarButtonBundle::default(),
        ],
    ));
}

#[derive(Bundle)]
/// A bundle to simplify the creation of toolbar buttons with predefined properties
struct ToolbarButtonBundle {
    node: Node,
    text: Text,
    border_radius: BorderRadius,
    border_color: BorderColor,
    background_color: BackgroundColor,
}

impl Default for ToolbarButtonBundle {
    fn default() -> Self {
        ToolbarButtonBundle {
            node: Node {
                height: Val::Px(TOOLBAR_SLOT_SIZE),
                width: Val::Px(TOOLBAR_SLOT_SIZE),
                border: UiRect::all(Val::Px(10.)),
                ..default()
            },
            text: Text::default(),
            border_radius: BorderRadius::all(Val::Px(5.)),
            border_color: BorderColor::from(GRAY_950),
            background_color: BackgroundColor::from(Srgba::new(0.0, 0.0, 0.0, 0.4)),
        }
    }
}
