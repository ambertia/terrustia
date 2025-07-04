use bevy::prelude::*;

use crate::player::{PLAYER_HEIGHT, Player};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, build_ui)
            .add_systems(Update, update_coordinates_ui);
    }
}

#[derive(Component)]
#[require(Text)]
struct UiCoordinateText;

fn update_coordinates_ui(
    mut text: Single<&mut Text, With<UiCoordinateText>>,
    player: Single<&Transform, With<Player>>,
) {
    text.0 = format!(
        "({0:.1}, {1:.1})",
        player.translation.x,
        player.translation.y - PLAYER_HEIGHT / 2.,
    );
}

fn build_ui(mut commands: Commands) {
    commands.spawn(UiCoordinateText);
}
