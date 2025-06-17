use bevy::{
    color::palettes::css::{CHOCOLATE, SADDLE_BROWN},
    math::{I16Vec2, bounding::Aabb2d},
    platform::collections::HashMap,
    prelude::*,
    window::PrimaryWindow,
};
use round_to::{CeilTo, FloorTo};

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameMap { 0: HashMap::new() })
            .add_event::<TileDestroyed>()
            .add_systems(Startup, build_terrain)
            .add_systems(FixedUpdate, (tile_interaction, tile_modifications).chain())
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

fn tile_interaction(
    mut tile_events: EventWriter<TileDestroyed>,
    camera: Single<(&Camera, &GlobalTransform)>,
    mouse: Res<ButtonInput<MouseButton>>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    // Only try to break blocks if the left mouse button is pressed
    if !mouse.pressed(MouseButton::Left) {
        return;
    }

    // Get the mouse position and convert to world space coordinates
    let cursor_pos = window.cursor_position().unwrap();
    let world_pos = camera.0.viewport_to_world_2d(camera.1, cursor_pos).unwrap();
    tile_events.write(TileDestroyed {
        position: I16Vec2::new(world_pos.x.floor_to(), world_pos.y.ceil_to()),
    });
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

const BLOCKS_X: i16 = 80;
const BLOCKS_Y: i16 = 40;
const BLOCK_SIZE: f32 = 10.;
/// Run on application setup to build the map data structure and spawn entities
fn build_terrain(mut game_map: ResMut<GameMap>, mut commands: Commands) {
    // Blocks are spawned from top-left to bottom-right. BLOCKS_X determines leftmost coordinate.
    for i in (-BLOCKS_X / 2)..(BLOCKS_X / 2) {
        for j in 0..(-1 * BLOCKS_Y) {
            commands.spawn((
                game_map.0.insert((i, j), TileData::default()).unwrap(), // TileData
                Sprite::default(),
                Transform {
                    translation: Vec3::new(f32::from(i), f32::from(j), 0.),
                    scale: Vec3::new(BLOCK_SIZE, BLOCK_SIZE, 0.0),
                    ..default()
                },
            ));
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
