use bevy::{math::bounding::Aabb2d, prelude::*};

#[derive(Resource)]
pub struct GameMap(HashMap<(i16, i16), TileData>);

impl GameMap {
    pub fn tile_at(&self, x: i16, y: i16) -> &TileData {
        &self.0[&(x, y)]
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
