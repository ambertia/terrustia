use bevy::{
    color::palettes::css::{CHOCOLATE, DEEP_SKY_BLUE, SADDLE_BROWN, SKY_BLUE},
    math::{I16Vec2, bounding::Aabb2d},
    platform::collections::HashMap,
    prelude::*,
    window::PrimaryWindow,
};
use round_to::{CeilTo, FloorTo};

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameMap>()
            .add_event::<TileDestroyed>()
            .add_observer(tile_destruction)
            .add_observer(tile_placement)
            .add_systems(Startup, build_terrain)
            .add_systems(FixedUpdate, tile_interaction)
            .add_systems(Update, tile_sprite_updates);
    }
}

/// Resource to associate tile entities in the ECS with map coordinates
#[derive(Resource, Default)]
pub struct GameMap(HashMap<(i16, i16), Entity>);

impl GameMap {
    /// Return the tile under a certain position in world space
    pub fn tile_under(&self, world_space: &Vec2) -> Option<Entity> {
        match self.0.get(&world_to_map_coord(world_space)) {
            Some(&e) => Some(e.to_owned()),
            None => None,
        }
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
            fg_id: 0,
            bg_id: 0,
            solid: false,
        }
    }
}

impl TileData {
    fn destroy(&mut self) -> usize {
        let old_tile_id = self.fg_id;
        self.fg_id = 0;
        self.solid = false;
        old_tile_id
    }

    pub fn is_solid(&self) -> bool {
        self.solid
    }
}

#[derive(Event)]
struct TileDestroyed;

#[derive(Event)]
struct TilePlaced;

/// Detect and trigger events on tiles by mouse input
fn tile_interaction(
    mut commands: Commands,
    camera: Single<(&Camera, &GlobalTransform)>,
    mouse: Res<ButtonInput<MouseButton>>,
    window: Single<&Window, With<PrimaryWindow>>,
    game_map: Res<GameMap>,
) {
    // Tile interaction can only occur when one of the mouse buttons is pressed
    if !mouse.any_pressed([MouseButton::Left, MouseButton::Right]) {
        return;
    }

    // Get the mouse position and convert to world space coordinates
    let cursor_pos = window.cursor_position().unwrap();
    let world_pos = camera.0.viewport_to_world_2d(camera.1, cursor_pos).unwrap();

    // Trigger Tile observers on the tile occupying those coordinates
    if let Some(t) = game_map.tile_under(&world_pos) {
        for button in mouse.get_pressed() {
            match button {
                // Entities implement Clone since they wrap an identifier for the ECS (like a key)
                MouseButton::Left => commands.trigger_targets(TileDestroyed, t),
                MouseButton::Right => commands.trigger_targets(TilePlaced, t),
                _ => continue,
            }
        }
    }
}

/// Modify tiles according to what happens in the world
fn tile_destruction(trigger: Trigger<TileDestroyed>, mut tiles: Query<&mut TileData>) {
    let mut tile = tiles.get_mut(trigger.target()).unwrap();
    tile.fg_id = 0;
    tile.solid = false;
}

fn tile_placement(trigger: Trigger<TilePlaced>, mut tiles: Query<&mut TileData>) {
    let mut tile = tiles.get_mut(trigger.target()).unwrap();
    tile.fg_id = 1;
    tile.solid = true;
}

/// Modify the Sprites of Entities with TileData Components that were just spawned or modified
fn tile_sprite_updates(tiles: Query<(&TileData, &mut Sprite), Changed<TileData>>) {
    // Right now tiles can be solid dirt, or background dirt. This means the logic for changing the
    // sprites can be very simple, but it will get complicated quickly as new blocks are added and
    // require referencing a resource of some kind.
    for tile in tiles {
        let (tile_data, mut sprite) = tile;
        if tile_data.fg_id == 0 {
            if tile_data.bg_id == 0 {
                sprite.color = Color::from(DEEP_SKY_BLUE);
            } else {
                sprite.color = Color::from(SADDLE_BROWN);
            }
        } else {
            sprite.color = Color::from(CHOCOLATE);
        }
    }
}

const BLOCKS_X: i16 = 80;
const BLOCKS_Y: i16 = 80;
const BLOCK_SIZE: f32 = 10.;
/// Run on application setup to build the map data structure and spawn tile entities
fn build_terrain(mut game_map: ResMut<GameMap>, mut commands: Commands) {
    // Blocks are spawned from bottom-left to top-right. BLOCKS_X determines leftmost coordinate.
    for i in (-BLOCKS_X / 2)..(BLOCKS_X / 2) {
        for j in (-BLOCKS_Y / 2)..(BLOCKS_Y / 2) {
            // Initial tile state depends on y value
            let tile_data = match j > 0 {
                true => TileData::default(),
                false => TileData {
                    fg_id: 1,
                    bg_id: 1,
                    solid: true,
                },
            };

            // Spawn tile in the world
            let tile_entity = commands
                .spawn((
                    tile_data,
                    Sprite::default(),
                    Transform {
                        translation: Vec3::new(
                            (f32::from(i) + 0.5) * BLOCK_SIZE,
                            (f32::from(j) - 0.5) * BLOCK_SIZE,
                            -1.,
                        ),
                        scale: Vec3::new(BLOCK_SIZE, BLOCK_SIZE, 1.),
                        ..default()
                    },
                ))
                .id();

            // Add the tile to the map resource
            game_map.0.insert((i, j), tile_entity);
        }
    }
}

/// Return a vec of all extant tile entities within a rectangular coordinate region
pub fn get_region_tiles(
    // TODO: This requires the user to pass in a reference to a Resource, which is clunky
    bottom_left: I16Vec2,
    top_right: I16Vec2,
    game_map: &GameMap,
) -> Vec<Entity> {
    let mut tiles: Vec<Entity> = Vec::new();

    for i in bottom_left.x..top_right.x {
        for j in bottom_left.y..top_right.y {
            if let Some(&e) = game_map.0.get(&(i, j)) {
                tiles.push(e.to_owned());
            }
        }
    }

    tiles
}

/// The range of tiles within which part of an object exists
/// This takes world space coordinates and returns map coordinates
pub fn occupied_tile_range(center: Vec2, size: Vec2) -> (I16Vec2, I16Vec2) {
    // Get the edges of the object in world space
    let top = (center.y + size.y / 2.0) / BLOCK_SIZE;
    let bottom = (center.y - size.y / 2.0) / BLOCK_SIZE;
    let right = (center.x + size.x / 2.0) / BLOCK_SIZE;
    let left = (center.x - size.x / 2.0) / BLOCK_SIZE;

    // Construct I16Vec2's representing map coordinates for the bottom-left and top-right tiles
    let bottom_left = I16Vec2::new(left.floor_to(), bottom.floor_to());
    let top_right = I16Vec2::new(right.ceil_to(), top.ceil_to());

    (bottom_left, top_right)
}

/// Return a bounding box in world space based on map coordinates
pub fn tile_coord_to_aabb2d(x: i16, y: i16) -> Aabb2d {
    Aabb2d::new(
        Vec2::new(
            f32::from(x) * BLOCK_SIZE + BLOCK_SIZE / 2.,
            f32::from(y) * BLOCK_SIZE + BLOCK_SIZE / 2.,
        ),
        Vec2::new(BLOCK_SIZE / 2., BLOCK_SIZE / 2.),
    )
}

pub fn world_to_map_coord(world_space: &Vec2) -> (i16, i16) {
    (
        (world_space.x / BLOCK_SIZE).floor_to(),
        (world_space.y / BLOCK_SIZE).ceil_to(),
    )
}
