use std::{
    cmp::{max, min},
    collections::VecDeque,
    error::Error,
    fmt::{self, Formatter},
};

use avian2d::prelude::*;
use bevy::{platform::collections::HashMap, prelude::*};

use super::{GameMap, TileData};

/// A struct containing map generation metadata
struct MapParameters {
    map_width: usize,
    map_height: usize,
    sky_height: i16,
    offsets_shift_limit: i16,
    offsets_run_min: usize,
    offsets_run_max: usize,
    hill_min_width: i16,
    hill_max_width: i16,
    hill_min_height: i16,
    hill_max_height: i16,
    hill_map_edge_margin: i16,
    hill_max_overlap: i16,
    hill_map_width_per: usize,
    dirt_thickness: i16,
    right_edge: i16,
    left_edge: i16,
    top_edge: i16,
    bottom_edge: i16,
}

// This takes some file constants and bakes them into map metadata
impl Default for MapParameters {
    fn default() -> Self {
        // Build a struct with all the manually defined parameters
        let mut params = MapParameters {
            map_width: 300,
            map_height: 50,
            sky_height: 15,
            offsets_shift_limit: 4,
            offsets_run_min: 5,
            offsets_run_max: 10,
            hill_min_width: 20,
            hill_max_width: 40,
            hill_min_height: 5,
            hill_max_height: 10,
            hill_map_edge_margin: 10,
            hill_max_overlap: 10,
            hill_map_width_per: 50,
            dirt_thickness: 5,
            right_edge: default(),
            left_edge: default(),
            top_edge: default(),
            bottom_edge: default(),
        };
        // Go over and actually compute the derived parameters (it's been convenient to have
        // these numbers on hand as i16)
        params.right_edge = i16::try_from(params.map_width / 2).unwrap();
        params.left_edge = -i16::try_from(params.map_width / 2).unwrap() - 1;
        params.top_edge = params.sky_height;
        params.bottom_edge = params.sky_height - i16::try_from(params.map_height).unwrap() + 1;

        // Return and transfer ownership of data
        params
    }
}

// This is where the high-level terrain generation control happens
impl FromWorld for GameMap {
    fn from_world(world: &mut World) -> Self {
        // Build a default of the map metadata struct
        let map_params = MapParameters::default();

        // Bake tile data after generating all additive features
        // There are two structures that affect the level of the ground and are necessary during
        // the "additive" phase of map generation - these are the "terrain offsets" and hills

        // Ground offsets are just a random variation intended to add subtle noise
        // Hills are geometric structures with width and height parameters
        let tile_data = rasterize_canvas(
            &map_params,
            generate_terrain_offsets(&map_params),
            generate_hills(&map_params),
        )
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

/// Generate a Vec containing the random offsets to ground height level
fn generate_terrain_offsets(params: &MapParameters) -> VecDeque<i16> {
    // Make an empty VecDeque that should be big enough for everything from the get-go
    let mut ground_offsets: VecDeque<i16> =
        VecDeque::with_capacity(params.map_width + params.offsets_run_max);
    // The actual value getting mutated when the terrain shifts
    let mut current_offset: i16 =
        rand::random_range(-params.offsets_shift_limit..=params.offsets_shift_limit);

    // Iterate across the map
    while ground_offsets.len() < params.map_width {
        // Pick a random length of blocks for this run
        let run_length = rand::random_range(params.offsets_run_min..=params.offsets_run_max);
        for _ in 0..run_length {
            // Strictly speaking, it doesn't matter if ground_offsets is a bit longer than
            // MAP_WIDTH - the extra offsets data just won't be used. This is why extra capacity is
            // allocated when the queue is initialized; this way there's no need for code to
            // awkwardly check if the run is going to push ground_offsets.len() over map_width when
            // picking a run_length or in this loop when pushing to ground_offsets.
            ground_offsets.push_back(current_offset);
        }

        // Shift with a weight that pushes back towards the center
        // The difference between max height and current height as a ratio
        let up_chance = f64::from(params.offsets_shift_limit - current_offset)
            / f64::from(2 * params.offsets_shift_limit);

        // Decide whether or not to shift up or down based on where in the height range we are
        if rand::random_bool(up_chance) {
            current_offset += 1;
        } else {
            current_offset -= 1;
        }

        // Clamp to within the allowable range (though due to the weighting this shouldn't
        // generally be necessary)
        current_offset =
            current_offset.clamp(-params.offsets_shift_limit, params.offsets_shift_limit);
    }
    ground_offsets
}

/// Data representation for hills used internally in terrain generation
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct HillParameters {
    x: i16,
    height: i16,
    width: i16,
}

impl HillParameters {
    /// Randomly generate a new hill with const-defined constraints
    fn new(params: &MapParameters) -> Self {
        HillParameters {
            x: rand::random_range(
                (params.left_edge + params.hill_map_edge_margin)
                    ..=(params.right_edge - params.hill_map_edge_margin),
            ),
            height: rand::random_range(params.hill_min_height..=params.hill_max_height),
            width: rand::random_range(params.hill_min_width..=params.hill_max_width),
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

const MAX_ATTEMPTS: usize = 50; // Kind of stinky way to prevent an infinite loop
/// Randomly generate all the hills necessary for the map
fn generate_hills(params: &MapParameters) -> Vec<HillParameters> {
    let hill_count = params.map_width / params.hill_map_width_per;
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
            if new_hill.get_overlap(&hill) > params.hill_max_overlap {
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

/// Bake all additive map generation data into TileData
fn rasterize_canvas(
    params: &MapParameters,
    mut offsets: VecDeque<i16>,
    hills: Vec<HillParameters>,
) -> Result<HashMap<(i16, i16), TileData>, BevyError> {
    // Safety checks
    // There have to be enough offsets for the size of the map; this shouldn't ever happen but it
    // would be a problem if it did.
    if offsets.len() < params.map_width {
        return Err(TerrainGenerationError(format!(
            "Offsets length {} is insufficent for map width {}",
            offsets.len(),
            params.map_width
        ))
        .into());
    }

    // Initialize the HashMap for block data. TileData will Default to an air block
    let mut map_data: HashMap<(i16, i16), TileData> =
        HashMap::with_capacity(params.map_width * params.map_height);

    // Iterate over the map from left to right
    for x in params.left_edge..=params.right_edge {
        // Determine the y-location of the grass block at the surface
        // Initialize a mut i16 using the random terrain offset
        let mut level = offsets.pop_front().unwrap_or_default();

        // Get the terrain offsets due to all hills at this location and add them to level
        for hill in &hills {
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
        for y in (level - params.dirt_thickness)..level {
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
        for y in params.bottom_edge..(level - params.dirt_thickness) {
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
