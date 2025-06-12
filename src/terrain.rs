use bevy::{
    color::palettes::css::{CHOCOLATE, SADDLE_BROWN},
    math::{I16Vec2, bounding::Aabb2d},
    platform::collections::HashMap,
    prelude::*,
};

use crate::BLOCK_SIZE;

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, tile_modifications)
            .add_systems(Update, tile_sprite_updates);
    }
}

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

#[derive(Event)]
struct TileDestroyed {
    position: I16Vec2,
}

/// Modify tiles according to what happens in the world
fn tile_modifications(mut tile_events: EventReader<TileDestroyed>, mut game_map: ResMut<GameMap>) {
    for event in tile_events.read() {
        game_map.destroy_at(event.position.x, event.position.y);
    }
}

/// Modify the Sprites of Entities with TileData Components that were just spawned or modified
fn tile_sprite_updates(tiles: Query<(&TileData, &mut Sprite), Changed<TileData>>) {
    // Right now tiles can be solid dirt, or background dirt. This means the logic for changing the
    // sprites can be very simple, but it will get complicated quickly as new blocks are added and
    // require referencing a resource of some kind.
    for tile in tiles {
        let (tile_data, mut sprite) = tile;
        if tile_data.solid {
            sprite.color = Color::from(CHOCOLATE);
        } else {
            sprite.color = Color::from(SADDLE_BROWN);
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
