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

#[derive(Default)]
struct CollidingOn {
    right: bool,
    left: bool,
    top: bool,
    bottom: bool,
}

// BUG: Player can get stuck in the floor slightly when hitting it at high speed, causing
// collisions with blocks in the ground when moving side-to-side
/// Check all movers for collisions with map tiles and alter their PhysicsBodys
fn check_collisions(
    movers: Query<(&mut PhysicsBody, &Transform)>,
    tiles: Query<(&TileData, &Transform)>,
    game_map: Res<GameMap>,
) {
    for mover in movers {
        // Track which edges of the mover are colliding with something
        let mut mover_collisions = CollidingOn::default();

        let (mut movement_state, transform) = mover;

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
            match get_collision(
                // Source
                Aabb2d::new(movement_state.position, transform.scale.truncate() / 2.),
                // Target
                Aabb2d::new(
                    tile_transform.translation.truncate(),
                    tile_transform.scale.truncate() / 2.,
                ),
            ) {
                Some(Collision::Right) => mover_collisions.right = true,
                Some(Collision::Left) => mover_collisions.left = true,
                Some(Collision::Top) => mover_collisions.top = true,
                Some(Collision::Bottom) => mover_collisions.bottom = true,
                None => continue,
            }
        }

        // Modify the mover's velocity based on edge conditions
        if mover_collisions.left && movement_state.velocity.x < 0. {
            movement_state.velocity.x *= -0.05;
        } else if mover_collisions.right && movement_state.velocity.x > 0. {
            movement_state.velocity.x *= -0.05;
        }

        if mover_collisions.bottom && movement_state.velocity.y < 0. {
            movement_state.velocity.y *= -0.05;
        } else if mover_collisions.top && movement_state.velocity.y > 0. {
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
fn tiles_occupied(mover: &PhysicsBody, transform: &Transform) -> (Range<i16>, Range<i16>) {
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
