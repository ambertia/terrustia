use std::ops::Range;

use bevy::math::bounding::{Aabb2d, BoundingVolume, IntersectsVolume};
use bevy::prelude::*;
use round_to::{CeilTo, FloorTo};

use crate::terrain::{GameMap, TileData, get_region_tiles, occupied_tile_range};
use crate::{BLOCK_SIZE, Player};

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                accel_env,
                accel_input,
                check_collisions_impulse,
                velocity_cap,
                position_update,
            )
                .chain(),
        )
        .add_systems(Update, transform_update);
    }
}

// Struct to contain physics data for moving entities
#[derive(Component)]
pub struct PhysicsBody {
    position: Vec2,
    velocity: Vec2,
    mass: f32,
}

impl PhysicsBody {
    pub fn from_pos(x: f32, y: f32) -> PhysicsBody {
        PhysicsBody {
            position: Vec2::new(x, y),
            ..default()
        }
    }
    fn apply_impulse(&mut self, impulse: Vec2) {
        let net_velocity = impulse / self.mass;
        self.velocity += net_velocity;
    }
}

impl Default for PhysicsBody {
    fn default() -> PhysicsBody {
        PhysicsBody {
            position: Vec2::ZERO,
            velocity: Vec2::ZERO,
            mass: 1.,
        }
    }
}

const DRAG_FACTOR: f32 = 0.05;
const GRAVITY: f32 = -15.;
/// Accelerate entities based on drag and gravity
fn accel_env(movers: Query<&mut PhysicsBody>, time_fixed: Res<Time<Fixed>>) {
    for mut mover in movers {
        let drag_impulse = mover.velocity * DRAG_FACTOR * time_fixed.delta_secs();
        let grav_impulse = GRAVITY * time_fixed.delta_secs();
        mover.velocity += drag_impulse + vec2(0., grav_impulse);
    }
}

const PLAYER_ACCEL: f32 = 60.;
/// Apply input-based acceleration to the character
fn accel_input(
    mut player: Single<&mut PhysicsBody, With<Player>>,
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
fn velocity_cap(movers: Query<&mut PhysicsBody>) {
    for mut mover in movers {
        if mover.velocity.length() < VELOCITY_MAX {
            continue;
        } else {
            let clamped = mover.velocity.clamp_length_max(VELOCITY_MAX);
            mover.velocity = clamped;
        }
    }
}

const IMPULSE_PER_OVERLAP: f32 = 0.1;
// Cap the impulse from overlap to two full block's worth
const COLLISION_IMPULSE_CAP: f32 = IMPULSE_PER_OVERLAP * BLOCK_SIZE * BLOCK_SIZE * 2.;
const CCD_THRESHOLD: f32 = 0.8;
fn check_collisions_impulse(
    movers: Query<(&mut PhysicsBody, &Transform)>,
    tiles: Query<(&TileData, &Transform)>,
    game_map: Res<GameMap>,
) {
    for mover in movers {
        let (mut physics_body, transform) = mover;

        // Make an Aabb2d for the mover so we don't have to do it in the loop below
        let mover_box = Aabb2d::new(physics_body.position, transform.scale.truncate() / 2.);

        // The range of tile coordinates the mover occupies depends on its position and size
        let (bottom_left, top_right) =
            occupied_tile_range(physics_body.position, transform.scale.truncate());

        // Get a Vec<Entity> for all extant nearby tiles
        let tile_entities = get_region_tiles(bottom_left, top_right, &game_map);

        // Mutables to track the total effect of terrain collisions on the mover
        let mut net_impulse: Vec2 = Vec2::ZERO;
        let mut net_overlap: f32 = 0.;

        // Iterate over all the nearby tiles
        for tile in tile_entities {
            // Get the tile's Query data
            let Ok((tile_data, tile_transform)) = tiles.get(tile) else {
                continue;
            };

            // Don't collide if the tile isn't solid
            if !tile_data.is_solid() {
                continue;
            };

            // Make an Aabb2d for the tile
            let tile_box = Aabb2d::new(
                tile_transform.translation.truncate(),
                tile_transform.scale.truncate() / 2.,
            );

            // Get the overlap area and direction from the mover's center to the tile's center
            let force_direction = (mover_box.center() - tile_box.center()).normalize();
            let overlap = get_overlap(mover_box, tile_box);

            // Update the total variables
            net_overlap += overlap;
            net_impulse += force_direction * overlap * IMPULSE_PER_OVERLAP;
        }

        // The net effect of all nearby tiles can now be applied to the mover
        physics_body.apply_impulse(net_impulse.clamp_length_max(COLLISION_IMPULSE_CAP));

        // Compare how much of the mover is actually overlapping with tiles. If above a certain
        // threshold, write an Event to trigger a primitive continuous-collision-detection
        // TODO: Implement CCD
        if net_overlap > mover_box.visible_area() * CCD_THRESHOLD {}
    }
}

// WARN: This can probably overflow f32's depending on what exactly the coordinates are, but with a
// map only going up to i16's it should be fine
/// Measure the overlap in world-space coordinates between two bounding boxes
fn get_overlap(source: Aabb2d, target: Aabb2d) -> f32 {
    // Special case
    if !source.intersects(&target) {
        return 0.;
    }

    // Two corners for each bounding box
    let source_bl = source.center() - source.half_size();
    let source_tr = source.center() + source.half_size();
    let target_bl = target.center() - target.half_size();
    let target_tr = target.center() + target.half_size();

    // Compute the corners of the overlap rectangle
    let overlap_bl = Vec2::new(source_bl.x.max(target_bl.x), source_bl.y.max(target_bl.y));
    let overlap_tr = Vec2::new(source_tr.x.min(target_tr.x), source_tr.x.min(target_tr.y));
    let side_lengths = overlap_tr - overlap_bl;
    side_lengths.x * side_lengths.y
}

/// Move entities in world space
fn position_update(movers: Query<&mut PhysicsBody>, time_fixed: Res<Time<Fixed>>) {
    for mut mover in movers {
        let position_delta = mover.velocity * time_fixed.delta_secs();
        mover.position += position_delta;
    }
}

/// Move entities in screen space
fn transform_update(movers: Query<(&PhysicsBody, &mut Transform)>, time_fixed: Res<Time<Fixed>>) {
    for mover in movers {
        let (state, mut transform) = mover;
        transform.translation =
            (state.position + state.velocity * time_fixed.overstep().as_secs_f32()).extend(0.0);
    }
}
