use std::{
    cmp::{max, min},
    collections::VecDeque,
    error::Error,
    fmt::{self, Formatter},
};

use avian2d::prelude::*;
use bevy::{platform::collections::HashMap, prelude::*};

use super::{GameMap, TileData};

const MAP_WIDTH: usize = 300;
const MAP_HEIGHT: usize = 50;

/// A struct containing map generation metadata
struct MapParameters {
    right_edge: i16,
    left_edge: i16,
    top_edge: i16,
    bottom_edge: i16,
}

// This takes some file constants and bakes them into map metadata
impl Default for MapParameters {
    fn default() -> Self {
        MapParameters {
            right_edge: i16::try_from(MAP_WIDTH / 2).unwrap(),
            left_edge: -i16::try_from(MAP_WIDTH / 2).unwrap() - 1,
            top_edge: SKY_HEIGHT,
            bottom_edge: SKY_HEIGHT - i16::try_from(MAP_HEIGHT).unwrap() + 1,
        }
    }
}

// This is where the high-level terrain generation control happens
impl FromWorld for GameMap {
    fn from_world(world: &mut World) -> Self {
        // Build a default of the map metadata struct
        let map_params = MapParameters::default();

        // There are two structures that affect the level of the ground and are necessary during
        // the "additive" phase of map generation - these are the "terrain offsets" and hills

        // Ground offsets are just a random variation intended to add subtle noise. If the method
        // fails, just default to all zeroes.
        let ground_offsets = match generate_terrain_offsets() {
            Ok(o) => o,
            Err(_) => VecDeque::from([0; MAP_WIDTH]),
        };

        // Hills are geometric structures with width and height parameters
        let hills = generate_hills(&map_params);

        // Bake everything we have so far into real block data
        let tile_data = rasterize_canvas(&map_params, ground_offsets, &hills)
            .expect("Failed to rasterize map canvas");

        // TODO: This is where the subtractive phase of terrain generation should occur, modifying
        // the raw block data in tile_data

        // Initialize the data structure for the GameMap resource itself once all the raw data is
        // done being modified
        let mut game_map: HashMap<(i16, i16), Entity> = HashMap::new();

        // Spawn tile entities here, while registering them in game_map
        for x in map_params.left_edge..=map_params.right_edge {
            for y in map_params.bottom_edge..=map_params.top_edge {
                // Retreive this tile's data from the baked data or default it for air blocks
                let data = match tile_data.get(&(x, y)) {
                    Some(td) => td,
                    None => &TileData::default(),
                };

                // The presence of a collider depends on whether or not the tile is solid
                let collider = match data.solid {
                    true => Some(Collider::rectangle(1., 1.)),
                    false => None,
                };

                // Spawn the tile entity and store its id in a variable
                let tile_entity = world
                    .commands()
                    .spawn((
                        data.to_owned(),
                        RigidBody::Static,
                        Sprite::default(),
                        Transform::from_xyz(f32::from(x) + 0.5, f32::from(y) - 0.5, -1.),
                    ))
                    .id();

                // Attach the collider if necessary
                if let Some(c) = collider {
                    world.commands().entity(tile_entity).insert(c);
                }

                // Register the tile entity's ID in the resource
                game_map.insert((x, y), tile_entity);
            }
        }
        GameMap(game_map)
    }
}

/// Custom error type implementing Error which wraps a String message
// NOTE: I could probably just use the simple_error crate for this, but it's fine for use here
#[derive(Debug)]
struct TerrainGenerationError(String);

impl fmt::Display for TerrainGenerationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for TerrainGenerationError {}

/// Maximum displacement from ground level; i.e. 4 -> Ground varies from -4 to 4
const SHIFT_LIMITER: i16 = 4;
const RUN_MIN: usize = 5;
const RUN_MAX: usize = 10; // This is a u8 because it needs conversion into both usize and i16
/// Generate a Vec containing the random offsets to ground height level
fn generate_terrain_offsets() -> Result<VecDeque<i16>, BevyError> {
    // TODO: I think I wrote this when MAP_WIDTH was i16. This is the only place I can see that
    // returns an error, and it's a conversion from usize to usize. I should remove this and have
    // the function return a VecDeque directly.
    let mut ground_offsets: VecDeque<i16> = VecDeque::with_capacity(MAP_WIDTH + RUN_MAX);
    let mut current_offset: i16 = rand::random_range(-SHIFT_LIMITER..=SHIFT_LIMITER);

    // Iterate across the map
    while ground_offsets.len() < MAP_WIDTH {
        // Pick a random length of blocks for the run at this height based on the constants
        let run_length = rand::random_range(RUN_MIN..=RUN_MAX);
        for _ in 0..run_length {
            // Strictly speaking, it doesn't matter if ground_offsets is a bit longer than
            // MAP_WIDTH - the extra offsets data just won't be used. This is why extra capacity is
            // allocated when the queue is initialized.
            ground_offsets.push_back(current_offset);
        }

        // Shift with a weight that pushes back towards the center
        // The difference between max height and current height as a ratio
        let up_chance = f64::from(SHIFT_LIMITER - current_offset) / f64::from(2 * SHIFT_LIMITER);

        // Decide whether or not to shift up or down based on where in the height range we are
        if rand::random_bool(up_chance) {
            current_offset += 1;
        } else {
            current_offset -= 1;
        }

        // Clamp to within the allowable range (though due to the weighting this shouldn't
        // generally be necessary)
        current_offset = current_offset.clamp(-SHIFT_LIMITER, SHIFT_LIMITER);
    }
    Ok(ground_offsets)
}

/// Data representation for hills used internally in terrain generation
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct HillParameters {
    x: i16,
    height: i16,
    width: i16,
}

const HILL_MAX_WIDTH: i16 = 40;
const HILL_MIN_WIDTH: i16 = 20;
const HILL_MAX_HEIGHT: i16 = 10;
const HILL_MIN_HEIGHT: i16 = 5;
const HILL_MAP_EDGE_MARGIN: i16 = 10;
impl HillParameters {
    /// Randomly generate a new hill with const-defined constraints
    fn new(params: &MapParameters) -> Self {
        HillParameters {
            x: rand::random_range(
                (params.left_edge + HILL_MAP_EDGE_MARGIN)
                    ..=(params.right_edge - HILL_MAP_EDGE_MARGIN),
            ),
            height: rand::random_range(HILL_MIN_HEIGHT..=HILL_MAX_HEIGHT),
            width: rand::random_range(HILL_MIN_WIDTH..=HILL_MAX_WIDTH),
        }
    }

    /// Calculate the overlap between this hill and another hill. Positive values indicate overlap,
    /// while negative values indicate a gap.
    fn get_overlap(&self, other: &HillParameters) -> i16 {
        // Find the right-most left edge and left-most right edge
        let left_bound = max(self.x - (self.width / 2), other.x - (other.width / 2));
        let right_bound = min(self.x + (self.width / 2), other.x + (other.width / 2));
        right_bound - left_bound
    }

    // Calculate the additional height provided by the hill at a given x-coordinate
    fn height_at(&self, x: i16) -> i16 {
        // Right now all hills are assumed to be triangular
        // Determine how far towards the the triangle's bottom corner this x-value is (symmetric)
        let dist_from_center = (self.x - x).abs();
        let height_raw = self.height - (self.height * dist_from_center / self.width);
        height_raw.clamp(0, self.height - 1)
    }
}

const HILL_MAX_OVERLAP: i16 = 10;
const WIDTH_PER_HILL: usize = 50; // Tiles of map width per hill generated
const MAX_ATTEMPTS: usize = 50; // Kind of stinky way to prevent an infinite loop
/// Randomly generate all the hills necessary for the map
fn generate_hills(params: &MapParameters) -> Vec<HillParameters> {
    let hill_count = MAP_WIDTH / WIDTH_PER_HILL;
    let mut hills: Vec<HillParameters> = Vec::new();
    let mut attempts: usize = 0;
    // This loop can O(n^2) relative to hill_count since it checks each new hill against each
    // existing hill at least once, possibly multiple times per new hill if it has to try again.
    'generation: while hills.len() < hill_count && attempts < MAX_ATTEMPTS {
        // Generate a hill
        let new_hill = HillParameters::new(params);
        attempts += 1;
        // Check it against all the existing hills
        for hill in hills.clone() {
            // If the new hill has unacceptable overlap with any hill, try a new one
            if new_hill.get_overlap(&hill) > HILL_MAX_OVERLAP {
                continue 'generation;
            }
        }
        // If this hill is compatible with all the others, push it to the Vec
        hills.push(new_hill);
    }
    // Sort the Vec left-to-right and return
    hills.sort();
    hills
}

/// How far above "ground level" the sky goes
const SKY_HEIGHT: i16 = 15;
/// How thick the layer of grass and dirt above the stone is
const DIRT_THICKNESS: i16 = 5;
/// Bake all additive map generation data into TileData
fn rasterize_canvas(
    params: &MapParameters,
    mut offsets: VecDeque<i16>,
    hills: &Vec<HillParameters>,
) -> Result<HashMap<(i16, i16), TileData>, BevyError> {
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

    // Initialize the HashMap for block data. TileData will Default to an air block
    let mut map_data: HashMap<(i16, i16), TileData> =
        HashMap::with_capacity(MAP_WIDTH * MAP_HEIGHT);

    // Iterate over the map from left to right
    for x in params.left_edge..=params.right_edge {
        // Determine the y-location of the grass block at the surface
        // Initialize a mut i16 using the random terrain offset
        let mut level = offsets.pop_front().unwrap_or_default();

        // Get the terrain offsets due to all hills at this location and add them to level
        for hill in hills {
            level += hill.height_at(x);
        }

        // Insert the grass block at (x, level)
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
        for y in params.bottom_edge..(level - DIRT_THICKNESS) {
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

    // Return the raw map tile data
    Ok(map_data)
}
