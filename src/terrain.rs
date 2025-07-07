use avian2d::prelude::{Collider, RigidBody};
use bevy::{
    color::palettes::tailwind::{
        AMBER_700, AMBER_900, CYAN_400, GREEN_700, NEUTRAL_950, STONE_500, STONE_700,
    },
    platform::collections::HashMap,
    prelude::*,
    time::Stopwatch,
    window::PrimaryWindow,
};
use round_to::{CeilTo, FloorTo};

use crate::inventory::ItemPickedUp;

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameMap>()
            .add_observer(tile_destruction)
            .add_observer(tile_placement)
            .add_systems(Startup, build_terrain)
            .add_systems(FixedUpdate, tile_interaction)
            .add_systems(Update, (tile_sprite_updates, tile_breaking_effect));
    }
}

/// Resource to associate tile entities in the ECS with map coordinates
#[derive(Resource, Default)]
pub struct GameMap(HashMap<(i16, i16), Entity>);

impl GameMap {
    /// Return the tile under a certain position in world space
    pub fn tile_under(&self, world_space: &Vec2) -> Option<Entity> {
        match self
            .0
            .get(&(world_space.x.floor_to(), world_space.y.ceil_to()))
        {
            Some(&e) => Some(e.to_owned()),
            None => None,
        }
    }
}

/// Contain the stateful data within a tile
#[derive(Component, Clone, Copy)]
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

// TODO: Do I want to save the partially-broken state of multiple tiles or just one? Terraria keeps
// that information for a short time - Maybe I should keep it for up to X tiles (e.g. 3-4?)
/// Component to help keep track of tile(s) currently being destroyed
#[derive(Component, Default)]
struct BreakTimer(Stopwatch);

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

const BREAK_TIME: f32 = 0.6;
/// Modify tiles according to what happens in the world. Player must hold the left mouse button
/// down over a period of time before the tile will actually break.
fn tile_destruction(
    trigger: Trigger<TileDestroyed>,
    mut tiles: Query<(&mut TileData, Option<&mut BreakTimer>)>,
    mut commands: Commands,
    time_fixed: Res<Time<Fixed>>,
    mut item_events: EventWriter<ItemPickedUp>,
) {
    let (mut tile, break_timer) = tiles.get_mut(trigger.target()).unwrap();

    // Tiles that aren't solid can't be broken
    if !tile.solid {
        return;
    }

    // Add a new timer to this tile if it's not already in the process of being broken
    // tile_interaction runs on FixedUpdate so use Time<Fixed> to advance stopwatches.
    // This observer will run at some arbitrary time after FixedUpdate, so use the
    // timestep() to advance rather than delta()
    let Some(mut break_timer) = break_timer else {
        let mut new_timer = BreakTimer::default();
        new_timer.0.tick(time_fixed.timestep());
        commands.entity(trigger.target()).insert(new_timer);
        return;
    };

    // Tick this tile's timer, but if it isn't ready yet don't destroy it
    break_timer.0.tick(time_fixed.timestep());
    if break_timer.0.elapsed_secs() < BREAK_TIME {
        return;
    }

    // Send the item to the player's inventory
    item_events.write(ItemPickedUp(tile.fg_id));

    // Modify the TileData and remove the BreakTimer component
    commands.entity(trigger.target()).remove::<BreakTimer>();
    tile.fg_id = 0;
    tile.solid = false;
    // Remove the tile's collider if present
    commands.entity(trigger.target()).remove::<Collider>();
}

fn tile_placement(
    trigger: Trigger<TilePlaced>,
    mut tiles: Query<&mut TileData>,
    mut commands: Commands,
) {
    let mut tile = tiles.get_mut(trigger.target()).unwrap();

    // Solid objects can't be placed on top of other solid objects
    if tile.solid {
        return;
    }

    tile.fg_id = 1;
    tile.solid = true;
    commands
        .entity(trigger.target())
        .insert(Collider::rectangle(1., 1.));
}

/// Modify the Sprites of Entities with TileData Components that were just spawned or modified
fn tile_sprite_updates(tiles: Query<(&TileData, &mut Sprite), Changed<TileData>>) {
    // TODO: The color picking by ID is only going to get worse
    for tile in tiles {
        let (tile_data, mut sprite) = tile;
        if tile_data.fg_id == 0 {
            if tile_data.bg_id == 0 {
                sprite.color = Color::from(CYAN_400);
            } else if tile_data.bg_id == 1 {
                sprite.color = Color::from(AMBER_900);
            } else {
                sprite.color = Color::from(STONE_700);
            }
        } else if tile_data.fg_id == 1 {
            sprite.color = Color::from(AMBER_700);
        } else if tile_data.fg_id == 2 {
            sprite.color = Color::from(GREEN_700);
        } else {
            sprite.color = Color::from(STONE_500);
        }
    }
}

/// Update a tile's sprite while it's being broken
fn tile_breaking_effect(tiles: Query<(&TileData, &BreakTimer, &mut Sprite), Changed<BreakTimer>>) {
    // TODO: More bad color picking by id that will only get worse
    for tile in tiles {
        let (tile_data, break_timer, mut sprite) = tile;

        let base_color = Color::from(match tile_data.fg_id {
            1 => AMBER_700,
            2 => GREEN_700,
            _ => STONE_500,
        });

        let breakage_frac = break_timer.0.elapsed_secs() / BREAK_TIME;
        sprite.color = base_color.mix(&Color::from(NEUTRAL_950), breakage_frac);
    }
}

const BLOCKS_X: i16 = 80;
const BLOCKS_Y: i16 = 80;
/// Run on application setup to build the map data structure and spawn tile entities
fn build_terrain(mut game_map: ResMut<GameMap>, mut commands: Commands) {
    // Blocks are spawned from bottom-left to top-right. BLOCKS_X determines leftmost coordinate.
    for i in (-BLOCKS_X / 2)..(BLOCKS_X / 2) {
        for j in (-BLOCKS_Y / 2)..(BLOCKS_Y / 2) {
            // Initial tile state depends on y value
            let tile_data = match j {
                1.. => TileData::default(),
                0 => TileData {
                    fg_id: 2,
                    bg_id: 1,
                    solid: true,
                },
                -10..0 => TileData {
                    fg_id: 1,
                    bg_id: 1,
                    solid: true,
                },
                ..-10 => TileData {
                    fg_id: 3,
                    bg_id: 3,
                    solid: true,
                },
            };

            // Presence of a collider depends on block state
            let collider = match j < 1 {
                true => Some(Collider::rectangle(1., 1.)),
                false => None,
            };

            // Spawn tile in the world
            let tile_entity = commands
                .spawn((
                    tile_data,
                    RigidBody::Static,
                    Sprite::default(),
                    Transform::from_xyz(f32::from(i) + 0.5, f32::from(j) - 0.5, -1.),
                ))
                .id();

            // Add the collider if the tile is solid
            if let Some(c) = collider {
                commands.entity(tile_entity).insert(c);
            }

            // Add the tile to the map resource
            game_map.0.insert((i, j), tile_entity);
        }
    }
}
