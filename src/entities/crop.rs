use std::collections::VecDeque;

use bevy::audio::PlaybackMode;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_ecs_tilemap::tiles::{TilePos, TileTextureIndex};
use rand::seq::IteratorRandom;

use crate::map::plugins::TilePosEvent;
use crate::map::tiled::{to_bevy_transform, LayerNumber, TiledMapInformation};

use super::streamer::StreamerLabel;
use crate::entities::TriggerQueue;

#[derive(Component, Debug, PartialEq, PartialOrd)]
pub enum CropState {
    Spawned,
    Planted,
    Growing,
    Grown,
}

#[derive(Component)]
pub struct CropEndIdx(usize);

#[derive(Event)]
pub struct NewSubscriber;

pub const CROP_NUM_STAGES: usize = 7;
const CROP_LAYER_NUM: usize = 13;
const IDEAL_CROP_LAYER_NUM: usize = 3;

pub fn replace_crop_tiles(
    tiles_query: Query<(Entity, &LayerNumber, &TilePos, &TileTextureIndex)>,
    map_info_query: Query<
        (&Transform, &TilemapGridSize, &TilemapSize, &TilemapType),
        Added<TilemapGridSize>,
    >,
    mut commands: Commands,
) {
    let map_information = map_info_query
        .iter()
        .find(|map_info| map_info.0.translation.z == IDEAL_CROP_LAYER_NUM as f32);

    if map_information.is_none() {
        return;
    }

    let (map_transform, grid_size, world_size, map_type) =
        map_information.expect("replace_crop_tiles: Map information should exist by now.");

    for (_entity, layer_number, tile_pos, tile_texture_index) in &tiles_query {
        if layer_number.0 != CROP_LAYER_NUM {
            continue;
        }

        let map_info = TiledMapInformation::new(grid_size, world_size, map_type, map_transform);
        let tile_transform = to_bevy_transform(tile_pos, map_info);

        commands.entity(_entity).despawn_recursive();
        commands.spawn((
            tile_transform,
            *tile_texture_index,
            *tile_pos,
            CropState::Spawned,
            CropEndIdx(tile_texture_index.0 as usize + CROP_NUM_STAGES - 1),
            TriggerQueue(VecDeque::new()),
        ));
    }
}

pub fn replace_crop_sprites(
    crops: Query<(Entity, &Transform, &TileTextureIndex), Added<CropState>>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    for (crop_entity, crop_transform, tile_texture_index) in &crops {
        let texture_handle = asset_server.load("environment/farming crops 1(16x16).png");
        let crop_texture_atlas =
            TextureAtlasLayout::from_grid(UVec2::new(16, 16), 16, 16, None, None);
        let crop_texture_atlas_handle = texture_atlases.add(crop_texture_atlas);

        let crop_texture_atlas = TextureAtlas {
            layout: crop_texture_atlas_handle.clone(),
            index: tile_texture_index.0 as usize,
        };

        let crop_sprite = SpriteBundle {
            sprite: Sprite::default(),
            texture: texture_handle.clone(),
            transform: *crop_transform,
            ..default()
        };

        commands.entity(crop_entity).remove::<Transform>();
        commands
            .entity(crop_entity)
            .insert((crop_sprite, crop_texture_atlas));
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

pub fn grow_crops(mut crop_query: Query<(&mut TriggerQueue, &mut CropState)>) {
    for (mut crop_queue, mut crop_state) in &mut crop_query {
        if !(*crop_state == CropState::Spawned
            || *crop_state == CropState::Planted
            || *crop_state == CropState::Growing)
        {
            continue;
        }

        if crop_queue.0.is_empty() {
            continue;
        }

        crop_queue.0.pop_front();
        *crop_state = CropState::Growing;
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
    mut crop_query: Query<(&mut CropState, &TilePos)>,
    streamer_query: Query<&TilePos, (With<StreamerLabel>, Changed<TilePos>)>,
) {
    if streamer_query.is_empty() {
        return;
    }

    let streamer_tile_pos = streamer_query.single();

    for (mut crop_state, crop_tile_pos) in &mut crop_query {
        if *crop_state != CropState::Grown {
            continue;
        }

        if streamer_tile_pos != crop_tile_pos {
            continue;
        }

        *crop_state = CropState::Planted;
    }
}

pub fn change_crop_sprite(
    mut crop_query: Query<(&mut TextureAtlas, &CropEndIdx, &mut CropState), Changed<CropState>>,
) {
    for (mut crop_texture_atlas, crop_end_idx, mut crop_state) in &mut crop_query {
        if *crop_state == CropState::Growing {
            crop_texture_atlas.index += 1;

            if crop_texture_atlas.index == crop_end_idx.0 {
                *crop_state = CropState::Grown;
            }
        } else if *crop_state == CropState::Planted {
            crop_texture_atlas.index -= CROP_NUM_STAGES - 1;
        }
    }
}

pub fn play_sound_for_crop(
    crop_query: Query<&CropState, Changed<TextureAtlas>>,
    asset_loader: Res<AssetServer>,
    mut commands: Commands,
) {
    for crop_state in &crop_query {
        // Planted in this case means it has been picked up by the Streamer.
        if *crop_state == CropState::Planted {
            let crop_pickedup_sound = AudioBundle {
                source: asset_loader.load("sfx/crop_pickedup.wav"),
                settings: PlaybackSettings {
                    mode: PlaybackMode::Despawn,
                    ..default()
                },
            };

            commands.spawn(crop_pickedup_sound);
        } else if *crop_state == CropState::Growing {
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
}
