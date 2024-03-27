use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use std::collections::VecDeque;

use crate::entities::streamer::{StreamerLabel, StreamerStatus};
use crate::map::path_finding::*;
use crate::map::tiled::{tiled_to_tile_pos, to_bevy_transform, LayerNumber, TiledMapInformation};
use crate::ui::chatting::Msg;

use super::MovementType;

pub const SUBSCRIBER_LAYER_NUM: usize = 19;
pub const DIST_AWAY_FROM_STREAMER: usize = 2;

#[derive(Component)]
pub struct SubscriberLabel;

#[derive(Component, PartialEq)]
pub enum SubscriberStatus {
    Idle,
    Approaching,
    Speaking,
    Leaving,
}

#[derive(Component)]
pub struct WaitTimer(Timer);

#[derive(Component, Event, Clone)]
pub struct SubscriberMsg {
    pub name: String,
    pub msg: String,
}

#[derive(Bundle)]
pub struct SubscriberBundle {
    label: SubscriberLabel,
    sprite: SpriteSheetBundle,
    movement_type: MovementType,
    status: SubscriberStatus,
}

pub fn replace_subscriber(
    mut tiles_query: Query<(Entity, &LayerNumber, &TilePos, &TileTextureIndex)>,
    map_info_query: Query<
        (&Transform, &TilemapGridSize, &TilemapSize, &TilemapType),
        Added<TilemapGridSize>,
    >,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let map_information = map_info_query
        .iter()
        .find(|map_info| map_info.0.translation.z == SUBSCRIBER_LAYER_NUM as f32);

    if map_information.is_none() {
        return;
    }

    let (map_transform, grid_size, map_size, map_type) =
        map_information.expect("replace_subscriber: Map information should exist by now.");

    let texture_handle = asset_server.load("subscribers/animation.png");
    let subscriber_texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::new(32.0, 32.0), 16, 16, None, None);
    let subscriber_texture_atlas_handle = texture_atlases.add(subscriber_texture_atlas);
    for (subscriber_entity, layer_number, tile_pos, tile_texture_index) in &mut tiles_query {
        if layer_number.0 != SUBSCRIBER_LAYER_NUM {
            continue;
        }

        let map_info = TiledMapInformation::new(grid_size, map_size, map_type, map_transform);
        let tile_transform = to_bevy_transform(tile_pos, map_info);

        let subscriber_sprite = SpriteSheetBundle {
            sprite: TextureAtlasSprite::new(tile_texture_index.0 as usize),
            texture_atlas: subscriber_texture_atlas_handle.clone(),
            transform: tile_transform,
            ..default()
        };
        let subscriber_tilepos = tiled_to_tile_pos(tile_pos.x, tile_pos.y, map_size);

        commands.entity(subscriber_entity).despawn_recursive();
        commands.spawn((
            SubscriberBundle {
                label: SubscriberLabel,
                sprite: subscriber_sprite,
                movement_type: MovementType::Swim,
                status: SubscriberStatus::Idle,
            },
            subscriber_tilepos,
        ));
    }
}
