use bevy::{math::bounding::Aabb2d, platform::collections::HashMap, prelude::*};

use crate::BLOCK_SIZE;

#[derive(Resource)]
pub struct GameMap(HashMap<(i16, i16), TileData>);

impl GameMap {
    /// Destroy a tile at given map coordinates and return its ID
    pub fn destroy_at(&mut self, x: i16, y: i16) -> usize {
        let tile = self.0.get_mut(&(x, y)).unwrap();
        let old_fg_id = tile.fg_id;
        tile.fg_id = 0;
        tile.solid = false;
        old_fg_id
    }
    pub fn solid_at(&self, x: i16, y: i16) -> bool {
        self.0.get(&(x, y)).unwrap().solid
    }
}

/// Contain the stateful data within a tile
#[derive(Component)]
pub struct TileData {
    fg_id: usize, // Foreground tile id
    bg_id: usize, // Background tile id
    solid: bool,  // Should entities collide with the tile?
}

impl Default for TileData {
    fn default() -> Self {
        TileData {
            fg_id: 1,
            bg_id: 1,
            solid: true,
        }
    }
}

/// Return a bounding box in world space based on map coordinates
pub fn map_space_to_aabb2d(x: i16, y: i16) -> Aabb2d {
    Aabb2d::new(
        Vec2::new(
            f32::from(x) + BLOCK_SIZE / 2.,
            f32::from(y) + BLOCK_SIZE / 2.,
        ),
        Vec2::new(BLOCK_SIZE / 2., BLOCK_SIZE / 2.),
    )
}
