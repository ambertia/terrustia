use std::ops::Range;

use bevy::math::bounding::{Aabb2d, BoundingVolume, IntersectsVolume};
use bevy::prelude::*;
use round_to::{CeilTo, FloorTo};

use crate::Player;
use crate::terrain::{GameMap, TileData, get_region_tiles, occupied_tile_range};

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                accel_env,
                accel_input,
                velocity_cap,
                check_collisions,
                position_update,
            )
                .chain(),
        )
        .add_systems(Update, transform_update);
    }
}

// Struct to contain physics data for moving entities
#[derive(Component, Default)]
pub struct MovementState {
    position: Vec2,
    velocity: Vec2,
}

impl MovementState {
    pub fn from_pos(x: f32, y: f32) -> MovementState {
        MovementState {
            position: Vec2::new(x, y),
            ..default()
        }
    }
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

fn check_collisions(
    movers: Query<(&mut MovementState, &Transform)>,
    tiles: Query<(&TileData, &Transform)>,
    game_map: Res<GameMap>,
) {
    for mover in movers {
        // Variables to track which edges of the mover are colliding with something
        let mut right = false;
        let mut left = false;
        let mut top = false;
        let mut bottom = false;

        let (mut movement_state, transform) = mover;

        // Collision side detection is affected by the relative dimensions of the mover
        let height_diff = (transform.scale.y - transform.scale.x) / 2.0;
        // The range of tile coordinates the mover occupies depends on its position and size
        let (bottom_left, top_right) =
            occupied_tile_range(movement_state.position, transform.scale.truncate());

        // Get a Vec<Entity> for all extant nearby tiles
        let tile_entities = get_region_tiles(bottom_left, top_right, &game_map);

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

            // Determine collision side using bounding boxes and mark accordingly
            let tile_box = Aabb2d::new(
                tile_transform.translation.truncate(),
                tile_transform.scale.truncate() / 2.0,
            );

            let offset = movement_state.position - tile_box.closest_point(movement_state.position);

            // To actually be touching, offsets must be within a certain range
            if offset.x.abs() > transform.scale.x / 2.0 {
                continue;
            } else if offset.y.abs() > transform.scale.y / 2.0 {
                continue;
            }

            if offset.x.abs() > offset.y.abs() - height_diff {
                if offset.x < 0.0 {
                    right = true;
                } else {
                    left = true;
                }
            } else if offset.y < 0.0 {
                top = true;
            } else {
                bottom = true;
            }
        }

        // Modify the mover's velocity based on edge conditions
        if left && movement_state.velocity.x < 0. {
            movement_state.velocity.x *= -0.05;
        } else if right && movement_state.velocity.x > 0. {
            movement_state.velocity.x *= -0.05;
        }

        if bottom && movement_state.velocity.y < 0. {
            movement_state.velocity.y *= -0.05;
        } else if top && movement_state.velocity.y > 0. {
            movement_state.velocity.y *= -0.05;
        }
    }
}

enum Collision {
    Right,
    Left,
    Top,
    Bottom,
}
/// Check for collisions between two bounding boxes and return depending on what side the collision
/// occurs on. source is the "subject" of the collision (i.e. player), and target is the object
/// being collided with (i.e. a block)
fn get_collision(source: Aabb2d, target: Aabb2d) -> Option<Collision> {
    if !source.intersects(&target) {
        return None;
    }

    let offset = source.center() - target.closest_point(source.center());

    // Get the margin. This implicitly compares how "deep" into the source the target closest point
    // is relative to its nearest corner.
    let margin = offset.abs() - source.half_size();

    // Object is colliding if margin points to the third quadrant (both negative)
    if margin.x > 0. || margin.y > 0. {
        return None;
    }

    // Determine what side the collision occurs on based on the offset and the relative height and
    // width of the source.
    let proportion = {
        let size = source.half_size();
        size.y / size.x
    };

    let side = if offset.x.abs() > offset.y.abs() / proportion {
        if offset.x < 0. {
            Collision::Right
        } else {
            Collision::Left
        }
    } else {
        if offset.y < 0. {
            Collision::Top
        } else {
            Collision::Bottom
        }
    };

    Some(side)
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
