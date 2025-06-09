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
// TODO: Accelerate entities based on drag and gravity
fn accel_env(movers: Query<&mut MovementState>, time_fixed: Res<Time<Fixed>>) {}
// TODO: Apply input-based acceleration to the character
fn accel_input(player: Single<&mut MovementState, With<Player>>, time_fixed: Res<Time<Fixed>>) {}

// TODO: Apply max speed to entities
fn velocity_cap(movers: Query<&mut MovementState>, time_fixed: Res<Time<Fixed>>) {}

// TODO: Check for collisions between entities and nearby solid objects
fn block_collisions(movers: Query<&mut MovementState>) {}

// TODO: Move entities
fn position_update(movers: Query<&mut MovementState>, time_fixed: Res<Time<Fixed>>) {}
