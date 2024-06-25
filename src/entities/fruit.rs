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

#[derive(Component, Deref, DerefMut)]
pub struct TriggerQueue(pub VecDeque<()>);

#[derive(Component, Debug, PartialEq, Eq)]
pub enum FruitState {
    Hanging,
    Falling,
    Dropped,
}

#[derive(Component)]
pub struct RespawnPoint(pub StartingPoint);

const FRUIT_LAYER_NUM: usize = 17;
const FALLEN_FRUIT_LAYER_NUM: usize = FRUIT_LAYER_NUM - 4;

pub fn replace_fruit_tiles(
    mut tiles_query: Query<(Entity, &LayerNumber, &TilePos, &TileTextureIndex)>,
    map_info_query: Query<
        (&Transform, &TilemapGridSize, &TilemapSize, &TilemapType),
        Added<TilemapGridSize>,
    >,
    mut commands: Commands,
) {
    let map_information = map_info_query
        .iter()
        .find(|map_info| map_info.0.translation.z == FRUIT_LAYER_NUM as f32);

    if map_information.is_none() {
        return;
    }

    let (map_transform, grid_size, map_size, map_type) =
        map_information.expect("replace_fruit_tiles: Map information should exist by now.");

    for (_entity, layer_number, tile_pos, tile_texture_index) in &mut tiles_query {
        if layer_number.0 != FRUIT_LAYER_NUM {
            continue;
        }

        let map_info = TiledMapInformation::new(grid_size, map_size, map_type, map_transform);
        let tile_transform = to_bevy_transform(tile_pos, map_info);

        commands.entity(_entity).despawn_recursive();
        commands.spawn((
            *tile_pos,
            *tile_texture_index,
            tile_transform,
            FruitState::Hanging,
            StartingPoint(tile_transform.translation, *tile_pos),
            RespawnPoint(StartingPoint(tile_transform.translation, *tile_pos)),
            Target(None),
            MovementTimer(Timer::from_seconds(0.05, TimerMode::Repeating)),
            TriggerQueue(VecDeque::new()),
        ));
    }
}

pub fn replace_fruit_sprites(
    fruit: Query<(Entity, &Transform, &TileTextureIndex), Added<FruitState>>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    for (fruit_entity, fruit_transform, tile_texture_index) in &fruit {
        let texture_handle = asset_server.load("environment/Fruit(16x16).png");
        let fruit_texture_atlas =
            TextureAtlasLayout::from_grid(Vec2::new(16.0, 16.0), 38, 6, None, None);
        let fruit_texture_atlas_handle = texture_atlases.add(fruit_texture_atlas);

        let fruit_sprite = SpriteSheetBundle {
            sprite: Sprite::default(),
            atlas: TextureAtlas {
                layout: fruit_texture_atlas_handle.clone(),
                index: tile_texture_index.0 as usize,
            },
            texture: texture_handle.clone(),
            transform: *fruit_transform,
            ..default()
        };

        commands.entity(fruit_entity).remove::<Transform>();
        commands.entity(fruit_entity).insert(fruit_sprite);
    }
}

pub fn make_fruit_fall(
    mut fruit_query: Query<(&TilePos, &mut FruitState, &mut Target, &TriggerQueue)>,
    ground_graph_query: Query<(&NodeData, &GraphType)>,
    map_info_query: Query<(&Transform, &TilemapSize)>,
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

    let map_information = map_info_query
        .iter()
        .find(|map_info| map_info.0.translation.z == FALLEN_FRUIT_LAYER_NUM as f32);

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
        let tile_translation: Vec3 = ground_graph_nodes.0
            [tilepos_to_idx(tile_target_pos.x, tile_target_pos.y, world_size.x)];
        let tile_transform = Transform::from_translation(tile_translation);

        fruit_pathing_target.0 = Some((tile_transform.translation, tile_target_pos));
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

pub fn respawn_fruit(
    mut fruit_query: Query<
        (
            &mut Transform,
            &mut TilePos,
            &mut StartingPoint,
            &RespawnPoint,
            &mut FruitState,
            &mut TriggerQueue,
        ),
        Without<AudioSink>,
    >,
    streamer_query: Query<&TilePos, (With<StreamerLabel>, Without<FruitState>, Changed<TilePos>)>,
) {
    if streamer_query.is_empty() {
        return;
    }

    let streamer_tilepos = streamer_query.single();
    for (
        mut fruit_transform,
        mut fruit_tilepos,
        mut fruit_starting_point,
        fruit_respawn_point,
        mut fruit_state,
        mut fruit_trigger_queue,
    ) in fruit_query.iter_mut()
    {
        if *fruit_state != FruitState::Dropped {
            continue;
        }

        if *streamer_tilepos != *fruit_tilepos {
            continue;
        }

        fruit_trigger_queue.0.pop_front();
        *fruit_tilepos = fruit_respawn_point.0 .1;
        *fruit_starting_point = StartingPoint(fruit_respawn_point.0 .0, fruit_respawn_point.0 .1);
        *fruit_transform = Transform::from_translation(fruit_respawn_point.0 .0);
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

/// Plays a sound depending on the changed state of the Fruit.
pub fn play_sound_for_fruit(
    fruit_query: Query<(Entity, &FruitState), Changed<FruitState>>,
    asset_loader: Res<AssetServer>,
    mut commands: Commands,
) {
    for (fruit_entity, fruit_state) in &fruit_query {
        match fruit_state {
            FruitState::Falling => {
                let fruit_fall_sound = AudioBundle {
                    source: asset_loader.load("sfx/fruit_dropped.wav"),
                    settings: PlaybackSettings {
                        mode: PlaybackMode::Remove,
                        ..default()
                    },
                };

                commands.entity(fruit_entity).insert(fruit_fall_sound);
            }
            FruitState::Hanging => {
                let fruit_pickedup_sound = AudioBundle {
                    source: asset_loader.load("sfx/fruit_pickedup.wav"),
                    settings: PlaybackSettings {
                        mode: PlaybackMode::Remove,
                        ..default()
                    },
                };

                commands.entity(fruit_entity).insert(fruit_pickedup_sound);
            }
            _ => continue,
        }
    }
}
