use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::map::tiled::LayerNumber;

use super::MovementType;

#[derive(Component)]
pub struct ChatterLabel;

#[derive(Bundle)]
pub struct ChatterBundle {
    label: ChatterLabel,
    sprite: SpriteSheetBundle,
    movement_type: MovementType,
}

pub fn replace_chatter(
    mut tiles_query: Query<(Entity, &LayerNumber, &TilePos, &TileTextureIndex)>,
    map_info_query: Query<(&Transform, &TilemapGridSize, &TilemapType), Added<TilemapGridSize>>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let chatter_tiles_layer_num = 18;

    let map_information = map_info_query
        .iter()
        .find(|map_info| map_info.0.translation.z == chatter_tiles_layer_num as f32);

    if map_information.is_none() {
        return;
    }

    let (map_transform, grid_size, map_type) =
        map_information.expect("replace_chatter: Map information should exist by now.");

    let texture_handle = asset_server.load("BirdSprite (16x16).png");
    let chatter_texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::new(16.0, 16.0), 8, 3, None, None);
    let chatter_texture_atlas_handle = texture_atlases.add(chatter_texture_atlas);
    for (crop_entity, layer_number, tile_pos, tile_texture_index) in &mut tiles_query {
        if layer_number.0 != chatter_tiles_layer_num {
            continue;
        }

        let tile_translation = tile_pos
            .center_in_world(grid_size, map_type)
            .extend(map_transform.translation.z);
        let tile_transform = *map_transform * Transform::from_translation(tile_translation);

        let chatter_sprite = SpriteSheetBundle {
            sprite: TextureAtlasSprite::new(tile_texture_index.0 as usize),
            texture_atlas: chatter_texture_atlas_handle.clone(),
            transform: tile_transform,
            ..default()
        };

        commands.entity(crop_entity).despawn_recursive();
        commands.spawn((
            ChatterBundle {
                label: ChatterLabel,
                sprite: chatter_sprite,
                movement_type: MovementType::Fly,
            },
            *tile_pos,
        ));
    }
}
