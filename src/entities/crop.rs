use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_ecs_tilemap::tiles::{TilePos, TileTextureIndex};
use rand::seq::IteratorRandom;

use crate::map::path_finding::{tilepos_to_idx, Path};
use crate::map::plugins::TilePosEvent;
use crate::map::tiled::{tiledpos_to_tilepos, LayerNumber};

use super::streamer::StreamerLabel;

#[derive(Component, PartialEq, PartialOrd)]
pub enum CropState {
    Growing,
    Grown,
    Ripe,
}

#[derive(Component)]
pub struct CropEndIdx(usize);

#[derive(Event)]
pub struct NewSubscriber;

const CROP_START_ROW_RANGE: [usize; 6] = [1, 3, 5, 7, 9, 11];
const CROP_COL_OFFSET: usize = 2;
const CROP_NUM_STAGES: usize = 7;

/// Maps a 2-dimensional (x, y) index into a 1-dimensional array index.
pub fn two_dim_to_one_dim_idx(x: usize, y: usize, num_cols: usize) -> usize {
    (num_cols * x) + y
}

pub fn replace_crop_tiles(
    mut tiles_query: Query<(Entity, &LayerNumber, &TilePos, &TileTextureIndex)>,
    map_info_query: Query<
        (&Transform, &TilemapGridSize, &TilemapSize, &TilemapType),
        Added<TilemapGridSize>,
    >,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
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
        TextureAtlas::from_grid(texture_handle, Vec2::new(16.0, 16.0), 16, 16, None, None);
    let crop_texture_atlas_handle = texture_atlases.add(crop_texture_atlas);
    for (_entity, layer_number, tile_pos, tile_texture_index) in &mut tiles_query {
        if layer_number.0 != crop_tiles_layer_num {
            continue;
        }

        let tile_translation = tile_pos
            .center_in_world(grid_size, map_type)
            .extend(map_transform.translation.z);
        let tile_transform = *map_transform * Transform::from_translation(tile_translation);

        let crop_sprite = SpriteSheetBundle {
            sprite: TextureAtlasSprite::new(tile_texture_index.0 as usize),
            texture_atlas: crop_texture_atlas_handle.clone(),
            transform: tile_transform,
            ..default()
        };

        commands.entity(_entity).despawn_recursive();
        commands.spawn((
            crop_sprite,
            tiledpos_to_tilepos(tile_pos.x, tile_pos.y, world_size),
            CropState::Growing,
            CropEndIdx(tile_texture_index.0 as usize + CROP_NUM_STAGES - 1),
        ));
    }
}

pub fn grow_crop_on_c_key(
    keyboard_input: Res<Input<KeyCode>>,
    mut subscriber_broadcaster: EventWriter<NewSubscriber>,
) {
    if keyboard_input.just_pressed(KeyCode::C) {
        subscriber_broadcaster.send(NewSubscriber);
    }
}

pub fn grow_crops(
    mut subscriber_reader: EventReader<NewSubscriber>,
    mut crop_query: Query<(&mut CropState, &mut TextureAtlasSprite, &mut CropEndIdx)>,
) {
    for _subscriber_msg in subscriber_reader.iter() {
        for (mut crop_state, mut crop_texture_atlas, crop_end_idx) in &mut crop_query {
            if *crop_state != CropState::Growing {
                continue;
            }

            crop_texture_atlas.index += 1;
            if crop_texture_atlas.index >= crop_end_idx.0 {
                *crop_state = CropState::Grown;
            }
        }
    }
}

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Visiting;

pub fn inform_streamer_of_grown_crops(
    mut crop_query: Query<(&mut CropState, &TilePos), Without<Visiting>>,
    mut streamer_destination_broadcast: EventWriter<TilePosEvent>,
) {
    let crop_to_pick_up = crop_query
        .iter_mut()
        .find(|crop_info| *crop_info.0 == CropState::Grown);

    if let Some(mut crop_tile_pos) = crop_to_pick_up {
        streamer_destination_broadcast.send(TilePosEvent::new(*crop_tile_pos.1, false));
    }
}

pub fn mark_crop_tile_visited(
    crop_query: Query<(Entity, &TilePos), (With<CropState>, Without<Visiting>)>,
    streamer_path_query: Query<&Path, (With<StreamerLabel>, Changed<Path>)>,
    map_info_query: Query<&TilemapSize>,
    mut commands: Commands,
) {
    if streamer_path_query.is_empty() {
        return;
    }

    let streamer_path = streamer_path_query.single();
    if streamer_path.back().is_none() {
        return;
    }

    let added_tile_pos = streamer_path
        .back()
        .expect("mark_crop_tile_visited: Streamer Path should have something.");

    let map_size = map_info_query
        .iter()
        .next()
        .expect("mark_crop_tile_visited: World should be populated by now.");
    if map_info_query.is_empty() {
        return;
    }

    let mut crop_indexes: Vec<(usize, Entity)> = crop_query
        .iter()
        .map(|crop_info| {
            (
                tilepos_to_idx(crop_info.1.x, crop_info.1.y, map_size.y),
                crop_info.0,
            )
        })
        .collect();

    crop_indexes.sort_unstable_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    let tile_found = crop_indexes.binary_search_by(|probe| probe.0.cmp(added_tile_pos));
    if let Ok(crop_tile_idx) = tile_found {
        let crop_entity = crop_indexes[crop_tile_idx].1;

        commands.entity(crop_entity).insert(Visiting);
    }
}

pub fn pick_up_crops(
    mut crop_query: Query<(
        Entity,
        &mut CropState,
        &mut TextureAtlasSprite,
        &mut CropEndIdx,
        &TilePos,
    )>,
    streamer_query: Query<&TilePos, (With<StreamerLabel>, Changed<TilePos>)>,
    mut commands: Commands,
) {
    if streamer_query.is_empty() {
        return;
    }

    let streamer_tile_pos = streamer_query.single();

    let crop_start_col_idx = CROP_COL_OFFSET;
    let crop_start_row_idx = *CROP_START_ROW_RANGE
        .iter()
        .choose(&mut rand::thread_rng())
        .expect("grow_crops: Could not pick a random column number for a crop.");

    for (crop_entity, mut crop_state, mut crop_texture_atlas, mut crop_end_idx, crop_tile_pos) in
        &mut crop_query
    {
        if *crop_state != CropState::Grown {
            continue;
        }

        if streamer_tile_pos != crop_tile_pos {
            continue;
        }

        crop_texture_atlas.index =
            two_dim_to_one_dim_idx(crop_start_row_idx, crop_start_col_idx, 16);
        crop_end_idx.0 = two_dim_to_one_dim_idx(
            crop_start_row_idx,
            crop_start_col_idx + CROP_NUM_STAGES - 1,
            16,
        );
        *crop_state = CropState::Growing;

        commands.entity(crop_entity).remove::<Visiting>();
    }
}