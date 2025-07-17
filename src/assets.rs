use bevy::prelude::*;

#[derive(Resource)]
pub struct TileAssets {
    pub handles: Vec<Handle<Image>>,
}
