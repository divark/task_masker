use bevy::prelude::*;
use bevy_ecs_tilemap::{
    prelude::{TilemapGridSize, TilemapSize, TilemapType},
    tiles::TilePos,
};

use crate::map::tiled::{tiled_to_bevy_transform, TiledMapInformation};

use super::MovementType;

#[derive(Component)]
pub struct StreamerLabel;

#[derive(Bundle)]
pub struct Streamer {
    label: StreamerLabel,
    sprites: SpriteSheetBundle,
    movement_type: MovementType,
}

pub fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    map_information: Query<
        (&Transform, &TilemapType, &TilemapGridSize, &TilemapSize),
        Added<TilemapType>,
    >,
    streamer_query: Query<(), With<StreamerLabel>>,
) {
    if !streamer_query.is_empty() {
        return;
    }

    let texture_handle = asset_server.load("caveman/caveman-sheet.png");
    let texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::new(16.0, 16.0), 4, 9, None, None);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    if map_information.is_empty() {
        return;
    }

    let (map_transform, map_type, grid_size, map_size) = map_information
        .iter()
        .nth(6)
        .expect("Could not load map information. Is world loaded?");
    let map_info = TiledMapInformation::new(grid_size, map_size, map_type, map_transform);

    let streamer_location = TilePos { x: 38, y: 59 }; //{ x: 42, y: 59 };
    let streamer_transform = tiled_to_bevy_transform(&streamer_location, map_info);

    commands.spawn((
        Streamer {
            label: StreamerLabel,
            sprites: SpriteSheetBundle {
                texture_atlas: texture_atlas_handle,
                transform: streamer_transform,
                ..default()
            },
            movement_type: MovementType::Walk,
        },
        streamer_location,
    ));
}
