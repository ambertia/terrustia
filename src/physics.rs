use std::ops::Range;
use std::time::Duration;

use bevy::{math::bounding::Aabb2d, prelude::*};
use round_to::{CeilTo, FloorTo};

use crate::terrain::GameMap;
use crate::{Player, terrain};

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                accel_env,
                accel_input,
                velocity_cap,
                block_collisions,
                position_update,
            )
                .chain(),
        )
        .add_systems(Update, transform_update);
    }
}

// Struct to contain physics data for moving entities
#[derive(Component, Default)]
struct MovementState {
    position: Vec2,
    velocity: Vec2,
}

const DRAG_FACTOR: f32 = 0.05;
const GRAVITY: f32 = -15.;
/// Accelerate entities based on drag and gravity
fn accel_env(movers: Query<&mut MovementState>, time_fixed: Res<Time<Fixed>>) {
    for mut mover in movers {
        let drag_impulse = mover.velocity * DRAG_FACTOR * time_fixed.delta_secs();
        let grav_impulse = GRAVITY * time_fixed.delta_secs();
        mover.velocity += drag_impulse + vec2(0., grav_impulse);
    }
}

const PLAYER_ACCEL: f32 = 60.;
/// Apply input-based acceleration to the character
fn accel_input(
    mut player: Single<&mut MovementState, With<Player>>,
    time_fixed: Res<Time<Fixed>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    let mut input_direction = Vec2::splat(0.0);

    // Check all the keys and modify the dot's input_direction accordingly
    if keyboard.pressed(KeyCode::KeyW) {
        input_direction.y += 1.0;
    }

    if keyboard.pressed(KeyCode::KeyA) {
        input_direction.x -= 1.0;
    }

    if keyboard.pressed(KeyCode::KeyS) {
        input_direction.y -= 1.0;
    }

    if keyboard.pressed(KeyCode::KeyD) {
        input_direction.x += 1.0;
    }

    // Apply an impulse to the player based on the inputs
    player.velocity += input_direction.normalize_or_zero() * PLAYER_ACCEL * time_fixed.delta_secs();
}

const VELOCITY_MAX: f32 = 300.;
/// Apply max speed to entities
fn velocity_cap(movers: Query<&mut MovementState>) {
    for mut mover in movers {
        if mover.velocity.length() < VELOCITY_MAX {
            continue;
        } else {
            let clamped = mover.velocity.clamp_length_max(VELOCITY_MAX);
            mover.velocity = clamped;
        }
    }
}

#[derive(Event)]
struct Collision(CollisionSide);

/// Represents what side of a moving object a collision occurs on
enum CollisionSide {
    Top,
    Bottom,
    Left,
    Right,
}

/// Check for collisions between entities and nearby solid objects, and fire a CollisionEvent
fn block_collisions(
    movers: Query<(&MovementState, &Transform)>,
    game_map: Res<GameMap>,
    mut events: EventWriter<Collision>,
) {
    for mover in movers {
        // Get the range of tile coordinates to check for blocks based on the mover's position and size
        let (movement_state, transform) = mover;
        let (range_x, range_y) = tiles_occupied(movement_state, transform);

        // Initialize a variable to track collision sides
        // TODO: What data structure to use to track collisions by side? Off-the-shelf Vecs would
        // provide x,y indexing, but that seems unnecessary when I could use a simple tuple instead
        let mut collision_directions: (i32, i32) = (0, 0);

        // Iterate over the nearby blocks
        // Have to Clone the ranges because they can't be Copy'd for implicit move (compiler whines)
        for x in range_x.clone() {
            for y in range_y.clone() {
                // Fetch tile data. Disregard this tile if it isn't collidable
                let tile = game_map.tile_at(x, y);
                if !tile.has_solid() {
                    continue;
                }

                // TODO: Analyze all nearby blocks before sending out a single Collision event for
                // the most "important" side - perhaps add the offsets together?
                // This would be fucky cause blocks that are worse offenders have smaller offsets

                // Change tuple depending on what side of the mover this block is colliding with
                let height_diff = (transform.scale.y - transform.scale.x) / 2.0;
                let offset = movement_state.position
                    - terrain::map_space_to_aabb2d(x, y).closest_point(movement_state.position);
                if offset.x.abs() > offset.y.abs() - height_diff {
                    if offset.x < 0.0 {
                        collision_directions.0 += 1;
                    } else {
                        collision_directions.0 -= 1;
                    }
                } else if offset.y < 0.0 {
                    collision_directions.1 += 1;
                } else {
                    collision_directions.1 -= 1;
                }

                // Fire a single Collision event based on the most "important" direction
                if collision_directions.0.abs() > collision_directions.1.abs() {
                    if collision_directions.0 > 0 {
                        events.write(Collision(CollisionSide::Right));
                    } else if collision_directions.0 < 0 {
                        events.write(Collision(CollisionSide::Left));
                    }
                } else if collision_directions.1 > 0 {
                    events.write(Collision(CollisionSide::Top));
                } else if collision_directions.1 < 0 {
                    events.write(Collision(CollisionSide::Bottom));
                }
            }
        }
    }
}

/// Return map coordinate ranges for all tiles at least partially occupied by the moving object.
fn tiles_occupied(mover: &MovementState, transform: &Transform) -> (Range<i16>, Range<i16>) {
    // Get corners of mover by adding scale to position
    let top_right: Vec2 = mover.position + transform.scale.truncate() / 2.;
    let bottom_left: Vec2 = mover.position - transform.scale.truncate() / 2.;
    // Use round_to functions to return Range<usize>'s
    return (
        bottom_left.x.floor_to()..top_right.x.floor_to(),
        bottom_left.y.ceil_to()..top_right.y.ceil_to(),
    );
}

/// Move entities in world space
fn position_update(movers: Query<&mut MovementState>, time_fixed: Res<Time<Fixed>>) {
    for mut mover in movers {
        let position_delta = mover.velocity * time_fixed.delta_secs();
        mover.position += position_delta;
    }
}

/// Move entities in screen space
fn transform_update(movers: Query<(&MovementState, &mut Transform)>, time_fixed: Res<Time<Fixed>>) {
    for mover in movers {
        let (state, mut transform) = mover;
        transform.translation =
            (state.position + state.velocity * time_fixed.overstep().as_secs_f32()).extend(0.0);
    }
}
