use bevy::{math::bounding::Aabb2d, prelude::*};

#[derive(Resource)]
pub struct GameMap(Vec<Vec<TileData>>);

impl GameMap {
    pub fn tile_at(&self, x: usize, y: usize) -> &TileData {
        &self.0[x][y]
    }
}

/// Contain the stateful data within a tile
#[derive(Component)]
pub struct TileData {
    fg_id: usize, // Foreground tile id
    bg_id: usize, // Background tile id
    solid: bool,  // Should entities collide with the tile?
}

impl TileData {
    pub fn has_solid(&self) -> bool {
        self.solid
    }
}

pub fn map_space_to_aabb2d(x: usize, y: usize) -> Aabb2d {}
