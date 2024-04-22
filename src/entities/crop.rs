use std::collections::VecDeque;

use bevy::audio::PlaybackMode;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_ecs_tilemap::tiles::{TilePos, TileTextureIndex};
use rand::seq::IteratorRandom;

use crate::map::plugins::TilePosEvent;
use crate::map::tiled::{tiled_to_tile_pos, to_bevy_transform, LayerNumber, TiledMapInformation};

use super::fruit::TriggerQueue;
use super::streamer::StreamerLabel;

#[derive(Component, PartialEq, PartialOrd)]
pub enum CropState {
    Growing,
    Grown,
}

#[derive(Component)]
pub struct CropEndIdx(usize);

#[derive(Event)]
pub struct NewSubscriber;

const CROP_NUM_STAGES: usize = 7;

pub fn replace_crop_tiles(
    mut tiles_query: Query<(Entity, &LayerNumber, &TilePos, &TileTextureIndex)>,
    map_info_query: Query<
        (&Transform, &TilemapGridSize, &TilemapSize, &TilemapType),
        Added<TilemapGridSize>,
    >,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let crop_tiles_layer_num = 13;

    let map_information = map_info_query
        .iter()
        .find(|map_info| map_info.0.translation.z == crop_tiles_layer_num as f32);

    if map_information.is_none() {
        return;
    }

    let (map_transform, grid_size, world_size, map_type) =
        map_information.expect("replace_crop_tiles: Map information should exist by now.");

    let texture_handle = asset_server.load("farming crops 1(16x16).png");
    let crop_texture_atlas =
        TextureAtlasLayout::from_grid(Vec2::new(16.0, 16.0), 16, 16, None, None);
    let crop_texture_atlas_handle = texture_atlases.add(crop_texture_atlas);
    for (_entity, layer_number, tile_pos, tile_texture_index) in &mut tiles_query {
        if layer_number.0 != crop_tiles_layer_num {
            continue;
        }

        let map_info = TiledMapInformation::new(grid_size, world_size, map_type, map_transform);
        let tile_transform = to_bevy_transform(tile_pos, map_info);

        let crop_sprite = SpriteSheetBundle {
            sprite: Sprite::default(),
            atlas: TextureAtlas {
                layout: crop_texture_atlas_handle.clone(),
                index: tile_texture_index.0 as usize,
            },
            texture: texture_handle.clone(),
            transform: tile_transform,
            ..default()
        };

        commands.entity(_entity).despawn_recursive();
        commands.spawn((
            crop_sprite,
            tiled_to_tile_pos(tile_pos.x, tile_pos.y, world_size),
            CropState::Growing,
            CropEndIdx(tile_texture_index.0 as usize + CROP_NUM_STAGES - 1),
            TriggerQueue(VecDeque::new()),
        ));
    }
}

pub fn grow_crop_on_c_key(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut crop_queues: Query<&mut TriggerQueue, With<CropState>>,
) {
    if crop_queues.is_empty() {
        return;
    }

    let mut random_crop_queue = crop_queues
        .iter_mut()
        .choose(&mut rand::thread_rng())
        .expect("Crop should exist by now.");

    if keyboard_input.just_pressed(KeyCode::KeyC) {
        random_crop_queue.0.push_back(());
    }
}

pub fn grow_crops(
    mut crop_query: Query<(
        &mut TriggerQueue,
        &mut CropState,
        &mut TextureAtlas,
        &mut CropEndIdx,
    )>,
    mut commands: Commands,
    asset_loader: Res<AssetServer>,
) {
    for (mut crop_queue, mut crop_state, mut crop_texture_atlas, crop_end_idx) in &mut crop_query {
        if *crop_state != CropState::Growing {
            continue;
        }

        if crop_queue.0.is_empty() {
            continue;
        }

        crop_queue.0.pop_front();

        crop_texture_atlas.index += 1;
        if crop_texture_atlas.index >= crop_end_idx.0 {
            *crop_state = CropState::Grown;
        }

        let crop_grow_sound = AudioBundle {
            source: asset_loader.load("sfx/crop_grow.wav"),
            settings: PlaybackSettings {
                mode: PlaybackMode::Despawn,
                ..default()
            },
        };

        commands.spawn(crop_grow_sound);
    }
}

pub fn pathfind_streamer_to_crops(
    crop_query: Query<(&CropState, &TilePos), Changed<CropState>>,
    mut streamer_destination_broadcast: EventWriter<TilePosEvent>,
) {
    for (crop_state, crop_tile_pos) in &crop_query {
        if *crop_state != CropState::Grown {
            continue;
        }

        streamer_destination_broadcast.send(TilePosEvent::new(*crop_tile_pos));
    }
}

pub fn pick_up_crops(
    mut crop_query: Query<(&mut CropState, &mut TextureAtlas, &TilePos)>,
    streamer_query: Query<&TilePos, (With<StreamerLabel>, Changed<TilePos>)>,
    mut commands: Commands,
    asset_loader: Res<AssetServer>,
) {
    if streamer_query.is_empty() {
        return;
    }

    let streamer_tile_pos = streamer_query.single();

    for (mut crop_state, mut crop_texture_atlas, crop_tile_pos) in &mut crop_query {
        if *crop_state != CropState::Grown {
            continue;
        }

        if streamer_tile_pos != crop_tile_pos {
            continue;
        }

        crop_texture_atlas.index -= CROP_NUM_STAGES - 1;
        *crop_state = CropState::Growing;

        let crop_pickedup_sound = AudioBundle {
            source: asset_loader.load("sfx/crop_pickedup.wav"),
            settings: PlaybackSettings {
                mode: PlaybackMode::Despawn,
                ..default()
            },
        };

        commands.spawn(crop_pickedup_sound);
    }
}
