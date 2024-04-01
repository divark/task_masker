use std::collections::VecDeque;

use crate::map::{
    path_finding::{tilepos_to_idx, GraphType, MovementTimer, NodeData, StartingPoint, Target},
    plugins::TilePosEvent,
    tiled::*,
};
use bevy::{audio::PlaybackMode, prelude::*};
use bevy_ecs_tilemap::prelude::*;
use rand::seq::IteratorRandom;

use super::streamer::StreamerLabel;

#[derive(Component)]
pub struct TriggerQueue(pub VecDeque<()>);

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
    map_info_query: Query<
        (&Transform, &TilemapGridSize, &TilemapSize, &TilemapType),
        Added<TilemapGridSize>,
    >,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let fruit_tiles_layer_num = 17;

    let map_information = map_info_query
        .iter()
        .find(|map_info| map_info.0.translation.z == fruit_tiles_layer_num as f32);

    if map_information.is_none() {
        return;
    }

    let (map_transform, grid_size, map_size, map_type) =
        map_information.expect("replace_fruit_tiles: Map information should exist by now.");

    let texture_handle = asset_server.load("Fruit(16x16).png");
    let fruit_texture_atlas =
        TextureAtlasLayout::from_grid(Vec2::new(16.0, 16.0), 38, 6, None, None);
    let fruit_texture_atlas_handle = texture_atlases.add(fruit_texture_atlas);
    for (_entity, layer_number, tile_pos, tile_texture_index) in &mut tiles_query {
        if layer_number.0 != fruit_tiles_layer_num {
            continue;
        }

        let map_info = TiledMapInformation::new(grid_size, map_size, map_type, map_transform);
        let tile_transform = to_bevy_transform(tile_pos, map_info);

        let fruit_sprite = SpriteSheetBundle {
            sprite: Sprite::default(),
            atlas: TextureAtlas {
                layout: fruit_texture_atlas_handle.clone(),
                index: tile_texture_index.0 as usize,
            },
            texture: texture_handle.clone(),
            transform: tile_transform,
            ..default()
        };

        commands.entity(_entity).despawn_recursive();
        commands.spawn((
            fruit_sprite,
            *tile_pos,
            FruitState::Hanging,
            StartingPoint(tile_transform.translation, *tile_pos),
            RespawnPoint(StartingPoint(tile_transform.translation, *tile_pos)),
            Target(None),
            MovementTimer(Timer::from_seconds(0.05, TimerMode::Repeating)),
            TriggerQueue(VecDeque::new()),
        ));
    }
}

pub fn make_fruit_fall(
    mut fruit_query: Query<(&TilePos, &mut FruitState, &mut Target, &TriggerQueue)>,
    ground_graph_query: Query<(&NodeData, &GraphType)>,
    map_info_query: Query<(&Transform, &TilemapSize)>,
    asset_loader: Res<AssetServer>,
    mut commands: Commands,
) {
    if ground_graph_query.is_empty() {
        return;
    }

    let ground_graph = ground_graph_query
        .iter()
        .find(|graph_elements| graph_elements.1 == &GraphType::Ground);
    if ground_graph.is_none() {
        return;
    }

    let ground_graph_nodes = ground_graph.unwrap().0;
    let fallen_fruit_tiles_layer_num = 16 - 4;

    let map_information = map_info_query
        .iter()
        .find(|map_info| map_info.0.translation.z == fallen_fruit_tiles_layer_num as f32);

    if map_information.is_none() {
        return;
    }

    let (_map_transform, world_size) =
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

        let tile_target_pos = TilePos::new(fruit_tile_pos.x + 3, fruit_tile_pos.y - 3);
        let tiled_target_pos = tiled_to_tile_pos(tile_target_pos.x, tile_target_pos.y, world_size);
        let tile_translation: Vec3 = ground_graph_nodes.0
            [tilepos_to_idx(tiled_target_pos.x, tiled_target_pos.y, world_size.x)];
        let tile_transform = Transform::from_translation(tile_translation);

        fruit_pathing_target.0 = Some((tile_transform.translation, tiled_target_pos));
        *fruit_state = FruitState::Falling;

        let fruit_fall_sound = AudioBundle {
            source: asset_loader.load("sfx/fruit_dropped.wav"),
            settings: PlaybackSettings {
                mode: PlaybackMode::Despawn,
                ..default()
            },
        };

        commands.spawn(fruit_fall_sound);
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
    fruit_query: Query<(&FruitState, &TilePos), Changed<FruitState>>,
    mut streamer_destination_request_writer: EventWriter<TilePosEvent>,
) {
    for (fruit_state, fruit_tile_pos) in fruit_query.iter() {
        if *fruit_state != FruitState::Dropped {
            continue;
        }

        streamer_destination_request_writer.send(TilePosEvent::new(*fruit_tile_pos));
    }
}

pub fn claim_fruit_from_streamer(
    mut fruit_query: Query<(&TilePos, &mut FruitState, &mut Visibility)>,
    streamer_query: Query<&TilePos, (With<StreamerLabel>, Changed<TilePos>)>,
    asset_loader: Res<AssetServer>,
    mut commands: Commands,
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

        let fruit_pickedup_sound = AudioBundle {
            source: asset_loader.load("sfx/fruit_pickedup.wav"),
            settings: PlaybackSettings {
                mode: PlaybackMode::Despawn,
                ..default()
            },
        };

        commands.spawn(fruit_pickedup_sound);
    }
}

pub fn respawn_fruit(
    mut fruit_query: Query<
        (
            &mut Transform,
            &mut TilePos,
            &mut StartingPoint,
            &RespawnPoint,
            &mut FruitState,
            &mut Visibility,
            &mut TriggerQueue,
        ),
        Changed<FruitState>,
    >,
) {
    for (
        mut fruit_transform,
        mut fruit_tile_pos,
        mut fruit_starting_point,
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
        *fruit_starting_point = StartingPoint(fruit_respawn_point.0 .0, fruit_respawn_point.0 .1);
        *fruit_transform = Transform::from_translation(fruit_respawn_point.0 .0);
        *fruit_sprite_visibility = Visibility::Visible;
        // Spawn Fruit Popping in Noise
        *fruit_state = FruitState::Hanging;
    }
}

pub fn drop_random_fruit_on_f_key(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut fruit_query: Query<&mut TriggerQueue, With<FruitState>>,
) {
    if fruit_query.is_empty() {
        return;
    }

    let mut random_fruit_queue = fruit_query
        .iter_mut()
        .choose(&mut rand::thread_rng())
        .expect("Fruit should exist.");

    if keyboard_input.just_pressed(KeyCode::KeyF) {
        random_fruit_queue.0.push_back(());
    }
}
