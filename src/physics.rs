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
        )
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

// TODO: Apply input-based acceleration to the character
fn accel_input(player: Single<&mut MovementState, With<Player>>, time_fixed: Res<Time<Fixed>>) {}

// TODO: Apply max speed to entities
fn velocity_cap(movers: Query<&mut MovementState>, time_fixed: Res<Time<Fixed>>) {}

// TODO: Check for collisions between entities and nearby solid objects
fn block_collisions(movers: Query<&mut MovementState>) {}

// TODO: Move entities
fn position_update(movers: Query<&mut MovementState>, time_fixed: Res<Time<Fixed>>) {}
