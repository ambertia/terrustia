use std::path::Path;

use bevy::prelude::*;

#[derive(Resource)]
pub struct TileAssets {
    pub handles: Vec<Handle<Image>>,
}

impl FromWorld for TileAssets {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let mut handles: Vec<Handle<Image>> = Vec::new();

        // Try to get an iterator over the folder's contents
        let Ok(rd) = Path::new("assets/sprites").read_dir() else {
            return Self { handles };
        };

        // Iterate over all the DirEntrys and add them to a Vec
        for file in rd {
            let Ok(f) = file else {
                continue;
            };
            // The file reference is a little weird but f.path() results in Bevy searching for the
            // assets in assets/assets/sprites/...
            handles.push(asset_server.load(Path::new("sprites/").join(f.file_name())));
        }

        Self { handles }
    }
}
