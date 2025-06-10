use bevy::prelude::*;

use crate::Player;

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
            ),
        );
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

    input_direction.normalize_or_zero();

    // Apply an impulse to the player based on the inputs
    player.velocity += input_direction * PLAYER_ACCEL * time_fixed.delta_secs();
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

// TODO: Check for collisions between entities and nearby solid objects
fn block_collisions(movers: Query<&mut MovementState>, game_map: Res<GameMap>) {
    for mut mover in movers {
        // Get the range of tile coordinates to check for blocks based on the mover's position and size
        // TODO: This relies on the player's size for now
        let (range_x, range_y) = mover.tiles_occupied();
        for x in range_x {
            for y in range_y {
                if !game_map.tile_at(x, y).has_solid() {
                    continue;
                }
            }
        }
    }
}

/// Move entities in world space
fn position_update(movers: Query<&mut MovementState>, time_fixed: Res<Time<Fixed>>) {
    for mut mover in movers {
        let position_delta = mover.velocity * time_fixed.delta_secs();
        mover.position += position_delta;
    }
}
