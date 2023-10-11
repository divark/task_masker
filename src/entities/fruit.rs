use std::collections::VecDeque;

use crate::map::{
    path_finding::{MovementTimer, StartingPoint, Target},
    tiled::*,
};
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use rand::seq::IteratorRandom;

use super::streamer::StreamerLabel;

#[derive(Component)]
pub struct TriggerQueue(VecDeque<()>);

#[derive(Component, PartialEq, Eq)]
pub enum FruitState {
    Hanging,
    Falling,
    Dropped,
    Claimed,
}

#[derive(Component)]
pub struct RespawnPoint(pub StartingPoint);

pub fn replace_fruit_tiles(
    mut tiles_query: Query<(Entity, &LayerNumber, &TilePos, &TileTextureIndex)>,
    map_info_query: Query<(&Transform, &TilemapGridSize, &TilemapType), Added<TilemapGridSize>>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let fruit_tiles_layer_num = 16;

    let map_information = map_info_query
        .iter()
        .find(|map_info| map_info.0.translation.z == fruit_tiles_layer_num as f32);

    if map_information.is_none() {
        return;
    }

    let (map_transform, grid_size, map_type) =
        map_information.expect("replace_fruit_tiles: Map information should exist by now.");

    let texture_handle = asset_server.load("Fruit(16x16).png");
    let fruit_texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::new(16.0, 16.0), 38, 6, None, None);
    let fruit_texture_atlas_handle = texture_atlases.add(fruit_texture_atlas);
    for (_entity, layer_number, tile_pos, tile_texture_index) in &mut tiles_query {
        if layer_number.0 != fruit_tiles_layer_num {
            continue;
        }

        let tile_translation = tile_pos
            .center_in_world(grid_size, map_type)
            .extend(map_transform.translation.z);
        let tile_transform = *map_transform * Transform::from_translation(tile_translation);

        let fruit_sprite = SpriteSheetBundle {
            sprite: TextureAtlasSprite::new(tile_texture_index.0 as usize),
            texture_atlas: fruit_texture_atlas_handle.clone(),
            transform: tile_transform,
            ..default()
        };

        commands.entity(_entity).despawn_recursive();
        commands.spawn((
            fruit_sprite,
            *tile_pos,
            FruitState::Hanging,
            StartingPoint(tile_translation, *tile_pos),
            RespawnPoint(StartingPoint(tile_translation, *tile_pos)),
            Target(None),
            MovementTimer(Timer::from_seconds(0.05, TimerMode::Repeating)),
            TriggerQueue(VecDeque::new()),
        ));
    }
}

pub fn make_fruit_fall(
    mut fruit_query: Query<(&TilePos, &mut FruitState, &mut Target, &TriggerQueue)>,
    map_info_query: Query<(&Transform, &TilemapGridSize, &TilemapType)>,
) {
    let fallen_fruit_tiles_layer_num = 16 - 4;

    let map_information = map_info_query
        .iter()
        .find(|map_info| map_info.0.translation.z == fallen_fruit_tiles_layer_num as f32);

    if map_information.is_none() {
        return;
    }

    let (map_transform, grid_size, map_type) =
        map_information.expect("make_fruit_fall: Map information should exist by now.");

    for (fruit_tile_pos, mut fruit_state, mut fruit_pathing_target, fruit_trigger_queue) in
        fruit_query.iter_mut()
    {
        if fruit_trigger_queue.0.is_empty() {
            continue;
        }

        if *fruit_state != FruitState::Hanging {
            continue;
        }

        let tile_translation: Vec3 = fruit_tile_pos
            .center_in_world(grid_size, map_type)
            .extend(map_transform.translation.z);
        let tile_transform = *map_transform * Transform::from_translation(tile_translation);

        fruit_pathing_target.0 = Some((
            tile_translation,
            TilePos::new(fruit_tile_pos.x + 3, fruit_tile_pos.y - 3),
        ));
        *fruit_state = FruitState::Falling;
    }
}

pub fn make_fruit_dropped(mut fruit_query: Query<(&mut FruitState, &Target)>) {
    for (mut fruit_state, fruit_pathing_target) in fruit_query.iter_mut() {
        if *fruit_state != FruitState::Falling {
            continue;
        }

        if fruit_pathing_target.is_some() {
            continue;
        }

        *fruit_state = FruitState::Dropped;
    }
}

pub fn pathfind_streamer_to_fruit(
    fruit_query: Query<(&FruitState, &TilePos, &Transform)>,
    mut streamer_query: Query<&mut Target, With<StreamerLabel>>,
) {
    if streamer_query.is_empty() {
        return;
    }

    let mut streamer_pathfinding_target = streamer_query.single_mut();
    if streamer_pathfinding_target.is_some() {
        return;
    }

    for (fruit_state, fruit_tile_pos, fruit_transform) in fruit_query.iter() {
        if *fruit_state != FruitState::Dropped {
            continue;
        }

        streamer_pathfinding_target.0 = Some((fruit_transform.translation, *fruit_tile_pos));
    }
}

pub fn claim_fruit_from_streamer(
    mut fruit_query: Query<(&TilePos, &mut FruitState, &mut Visibility)>,
    streamer_query: Query<&TilePos, With<StreamerLabel>>,
) {
    if streamer_query.is_empty() {
        return;
    }

    let streamer_tile_pos = streamer_query.single();
    for (fruit_tile_pos, mut fruit_state, mut fruit_sprite_visibility) in fruit_query.iter_mut() {
        if *fruit_state != FruitState::Dropped {
            continue;
        }

        if streamer_tile_pos != fruit_tile_pos {
            continue;
        }

        *fruit_state = FruitState::Claimed;
        *fruit_sprite_visibility = Visibility::Hidden;
        // Spawn Fruit Claimed noise here.
    }
}

pub fn respawn_fruit(
    mut fruit_query: Query<(
        &mut Transform,
        &mut TilePos,
        &RespawnPoint,
        &mut FruitState,
        &mut Visibility,
        &mut TriggerQueue,
    )>,
) {
    for (
        mut fruit_transform,
        mut fruit_tile_pos,
        fruit_respawn_point,
        mut fruit_state,
        mut fruit_sprite_visibility,
        mut fruit_trigger_queue,
    ) in fruit_query.iter_mut()
    {
        if *fruit_sprite_visibility != Visibility::Hidden {
            continue;
        }

        fruit_trigger_queue.0.pop_front();
        *fruit_tile_pos = fruit_respawn_point.0 .1;
        *fruit_transform = Transform::from_translation(fruit_respawn_point.0 .0);
        *fruit_sprite_visibility = Visibility::Visible;
        // Spawn Fruit Popping in Noise
        *fruit_state = FruitState::Hanging;
    }
}

pub fn drop_random_fruit_on_f_key(
    keyboard_input: Res<Input<KeyCode>>,
    mut fruit_query: Query<(&mut TriggerQueue,)>,
) {
    if fruit_query.is_empty() {
        return;
    }

    let mut random_fruit_queue = fruit_query
        .iter_mut()
        .choose(&mut rand::thread_rng())
        .expect("Fruit should exist.")
        .0;

    if keyboard_input.just_pressed(KeyCode::F) {
        random_fruit_queue.0.push_back(());
    }
}
