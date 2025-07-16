use std::collections::VecDeque;
use std::fmt;
use std::{error::Error, fmt::Formatter};

use avian2d::prelude::*;
use bevy::{platform::collections::HashMap, prelude::*};

use super::{GameMap, TileData};

const MAP_WIDTH: usize = 200;
const MAP_HEIGHT: usize = 40;

struct MapParameters {
    right_edge: i16,
    left_edge: i16,
    top_edge: i16,
    bottom_edge: i16,
}

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

impl FromWorld for GameMap {
    fn from_world(world: &mut World) -> Self {
        let map_params = MapParameters::default();

        let ground_offsets = match generate_terrain_offsets() {
            Ok(o) => o,
            Err(_) => VecDeque::from([0; MAP_WIDTH]),
        };

        // Bake the parameters into real TileData
        let tile_data =
            rasterize_canvas(ground_offsets, &map_params).expect("Failed to rasterize map canvas");

        let mut game_map: HashMap<(i16, i16), Entity> = HashMap::new();
        // Spawn the tiles here and add them to game_map
        for x in map_params.left_edge..=map_params.right_edge {
            for y in map_params.bottom_edge..=map_params.top_edge {
                // Retreive this tile's data from the final collection or default it
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

                // Attach the collider if it exists
                if let Some(c) = collider {
                    world.commands().entity(tile_entity).insert(c);
                }

                game_map.insert((x, y), tile_entity);
            }
        }
        GameMap(game_map)
    }
}

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
fn generate_terrain_offsets() -> Result<VecDeque<i16>, BevyError> {
    let mut ground_offsets: VecDeque<i16> = VecDeque::with_capacity(MAP_WIDTH.try_into()?);
    let mut running_offset: i16 = rand::random_range(-SHIFT_LIMITER..=SHIFT_LIMITER);
    ground_offsets.push_back(running_offset);

    // Iterate across the map
    for _ in 1..MAP_WIDTH {
        // Only once every SHIFT_INTERVAL blocks on average should the terrain height shift
        if !rand::random_bool(SHIFT_CHANCE) {
            ground_offsets.push_back(running_offset);
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

        // Push this value to the VecDeque
        ground_offsets.push_back(running_offset);
    }
    Ok(ground_offsets)
}

/// How far above "ground level" the sky goes
const SKY_HEIGHT: i16 = 10;
/// How thick the layer of grass and dirt above the stone is
const DIRT_THICKNESS: i16 = 5;
/// Construct basic terrain map data by taking into account random terrain offset
fn rasterize_canvas(
    mut offsets: VecDeque<i16>,
    params: &MapParameters,
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
        HashMap::with_capacity((MAP_WIDTH * MAP_HEIGHT).try_into()?);

    // Iterate over the map lengthwise
    for x in params.left_edge..=params.right_edge {
        // Determine the y-location of the grass block at the surface
        // Offsets Vec is 0-indexed, which makes this more complicated
        let level = offsets.pop_front().unwrap_or_default();

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

    Ok(map_data)
}
