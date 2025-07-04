use avian2d::{math::Vector, prelude::*};
use bevy::prelude::*;

use crate::inventory::Inventory;

pub struct CharacterControllerPlugin;

impl Plugin for CharacterControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, ((update_grounded, keyboard_input).chain(),))
            .add_systems(Startup, spawn_player);
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

pub const PLAYER_WIDTH: f32 = 2.;
pub const PLAYER_HEIGHT: f32 = 3.;
fn spawn_player(mut commands: Commands) {
    commands.spawn((
        Player,
        RigidBody::Dynamic,
        Collider::rectangle(PLAYER_WIDTH - 0.1, PLAYER_HEIGHT - 0.1),
        Sprite {
            color: Color::from(Srgba::new(1., 1., 1., 1.)),
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
        Inventory::default(),
    ));
}
