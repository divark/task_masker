use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::entities::streamer::StreamerLabel;
use crate::entities::GameEntityType;
use crate::map::path_finding::Direction;
use crate::map::tiled::{to_bevy_transform, LayerNumber, TiledMapInformation};

pub const CAMPFIRE_LAYER_NUM: usize = 20;

#[derive(Component)]
pub struct CampfireLabel;

/// Respawns Campfire with rendering components
pub fn replace_campfire_sprite(
    campfire: Query<(Entity, &Transform, &TileTextureIndex), Added<CampfireLabel>>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    for (campfire_entity, campfire_transform, tile_texture_index) in &campfire {
        let texture_handle =
            asset_server.load("environment/Sprite-sheet-campfire-trimmed-64x64.png");
        let campfire_texture_atlas =
            TextureAtlasLayout::from_grid(UVec2::new(64, 64), 23, 1, None, None);
        let campfire_texture_atlas_handle = texture_atlases.add(campfire_texture_atlas);

        let campfire_texture_atlas = TextureAtlas {
            layout: campfire_texture_atlas_handle.clone(),
            index: tile_texture_index.0 as usize,
        };

        let campfire_sprite = SpriteBundle {
            sprite: Sprite::default(),
            texture: texture_handle.clone(),
            transform: *campfire_transform,
            ..default()
        };

        commands.entity(campfire_entity).remove::<Transform>();
        commands
            .entity(campfire_entity)
            .insert((campfire_sprite, campfire_texture_atlas));
    }
}

/// Respawns Campfire without rendering components
pub fn replace_campfire_tile(
    tiles_query: Query<(Entity, &LayerNumber, &TilePos, &TileTextureIndex)>,
    map_info_query: Query<
        (&Transform, &TilemapGridSize, &TilemapSize, &TilemapType),
        Added<TilemapGridSize>,
    >,
    mut commands: Commands,
) {
    let map_information = map_info_query
        .iter()
        .find(|map_info| map_info.0.translation.z == CAMPFIRE_LAYER_NUM as f32);

    if map_information.is_none() {
        return;
    }

    let (map_transform, grid_size, map_size, map_type) =
        map_information.expect("replace_campfire_tile: Map information should exist by now.");

    for (campfire_entity, layer_number, tile_pos, tile_texture_index) in &tiles_query {
        if layer_number.0 != CAMPFIRE_LAYER_NUM {
            continue;
        }

        let map_info = TiledMapInformation::new(grid_size, map_size, map_type, map_transform);
        let tile_transform = to_bevy_transform(tile_pos, map_info);

        commands.entity(campfire_entity).despawn_recursive();
        commands.spawn((
            (
                CampfireLabel,
                tile_transform,
                GameEntityType::Environment,
                *tile_texture_index,
            ),
            *tile_pos,
        ));
    }
}

/// Makes the Streamer face towards the Campfire when
/// next to it.
pub fn make_streamer_face_campfire(
    mut streamer_query: Query<(&TilePos, &mut Direction), (With<StreamerLabel>, Changed<TilePos>)>,
    campfire: Query<&TilePos, With<CampfireLabel>>,
) {
    if streamer_query.is_empty() || campfire.is_empty() {
        return;
    }

    let (streamer_tilepos, mut streamer_direction) = streamer_query.single_mut();
    let campfire_tilepos = campfire.single();
    let left_of_campfire = TilePos::new(campfire_tilepos.x, campfire_tilepos.y - 1);

    if *streamer_tilepos != left_of_campfire {
        return;
    }

    *streamer_direction = Direction::BottomRight;
}
