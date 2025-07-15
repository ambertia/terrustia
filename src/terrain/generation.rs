use std::fmt;
use std::{error::Error, fmt::Formatter};

use bevy::{platform::collections::HashMap, prelude::*};

use super::{GameMap, TileData};

const MAP_WIDTH: usize = 200;
const MAP_HEIGHT: usize = 30;

/// Custom error type implementing Error which wraps a String message
// TODO: I could probably just use the simple_error crate for this, but it's fine for use here
#[derive(Debug)]
struct TerrainGenerationError(String);

impl fmt::Display for TerrainGenerationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for TerrainGenerationError {}

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

/// How far above "ground level" the sky goes
const SKY_HEIGHT: i16 = 10;
/// How thick the layer of grass and dirt above the stone is
const DIRT_THICKNESS: i16 = 5;
/// Construct basic terrain map data by taking into account random terrain offset
fn rasterize_canvas(offsets: Vec<i16>) -> Result<HashMap<(i16, i16), TileData>, Box<dyn Error>> {
    // Safety checks
    // There have to be enough offsets for the size of the map; this shouldn't ever happen but it
    // would be a problem if it did.
    if offsets.len() < MAP_WIDTH {
        return Err(TerrainGenerationError(format!(
            "Offsets length {} is insufficent for map width {}",
            offsets.len(),
            MAP_WIDTH
        ))
        .into());
    }

    // Basic parameters to use later
    let right_edge: i16 = i16::try_from(MAP_WIDTH)? / 2;
    let left_edge: i16 = right_edge - i16::try_from(MAP_WIDTH)?;
    let bottom_edge: i16 = SKY_HEIGHT - i16::try_from(MAP_HEIGHT)? + 1;

    // Initialize the HashMap for block data. TileData will Default to an air block
    let mut map_data: HashMap<(i16, i16), TileData> =
        HashMap::with_capacity(MAP_WIDTH * MAP_HEIGHT);

    // Iterate over the map lengthwise
    for x in left_edge..=right_edge {
        // Determine the y-location of the grass block at the surface
        // Offsets Vec is 0-indexed, which makes this more complicated
        let level = offsets[usize::try_from(x + left_edge)?];

        // Insert the dirt block at (x, level)
        map_data.insert(
            (x, level),
            TileData {
                fg_id: 2,
                bg_id: 1,
                solid: true,
            },
        );

        // Insert dirt tiles underneath the grass block until DIRT_THICKNESS tiles have been placed
        for y in (level - DIRT_THICKNESS)..level {
            map_data.insert(
                (x, y),
                TileData {
                    fg_id: 1,
                    bg_id: 1,
                    solid: true,
                },
            );
        }

        // Insert stone tiles from the bottom of the map to the dirt layer
        for y in bottom_edge..(level - DIRT_THICKNESS) {
            map_data.insert(
                (x, y),
                TileData {
                    fg_id: 3,
                    bg_id: 3,
                    solid: true,
                },
            );
        }
    }

    Ok(map_data)
}
