use bevy::{platform::collections::HashMap, prelude::*};

use super::{GameMap, TileData};

const MAP_WIDTH: usize = 200;
const MAP_HEIGHT: usize = 30;

/// Probability of the terrain shifting height each tile
const SHIFT_CHANCE: f64 = 0.125;
/// Maximum displacement from ground level; i.e. 4 -> Ground varies from -4 to 4
const SHIFT_LIMITER: i16 = 4;
/// Generate a Vec containing the random offsets to ground height level
fn generate_terrain_offsets() -> Vec<i16> {
    let mut ground_offsets: Vec<i16> = Vec::with_capacity(MAP_WIDTH);
    let mut running_offset: i16 = rand::random_range(-SHIFT_LIMITER..=SHIFT_LIMITER);
    ground_offsets[0] = running_offset;

    // Iterate across the map
    for i in 1..MAP_WIDTH {
        // Only once every SHIFT_INTERVAL blocks on average should the terrain height shift
        if !rand::random_bool(SHIFT_CHANCE) {
            ground_offsets[i] = running_offset;
            continue;
        }

        // Shift with a weight that pushes back towards the center
        // The difference between max height and current height as a ratio
        let up_chance = f64::from(SHIFT_LIMITER - running_offset) / f64::from(2 * SHIFT_LIMITER);

        // Decide whether or not to shift up or down based on where in the height range we are
        if rand::random_bool(up_chance) {
            running_offset += 1;
        } else {
            running_offset -= 1;
        }

        // Clamp to within the allowable range (though due to the weighting this shouldn't
        // generally be necessary)
        running_offset = running_offset.clamp(-SHIFT_LIMITER, SHIFT_LIMITER);

        // Assign to the current index
        ground_offsets[i] = running_offset
    }
    ground_offsets
}
