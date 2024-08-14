use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use std::collections::VecDeque;

use crate::entities::streamer::{StreamerLabel, StreamerState};
use crate::entities::WaitTimer;
use crate::map::path_finding::*;
use crate::map::tiled::{to_bevy_transform, LayerNumber, TiledMapInformation};
use crate::ui::chatting::Msg;

use super::GameEntityType;

pub const SUBSCRIBER_LAYER_NUM: usize = 18;
pub const DESIRED_SUBSCRIBER_LAYER_NUM: usize = 1;

#[derive(Component)]
pub struct SubscriberLabel;

#[derive(Component, Debug, PartialEq)]
pub enum SubscriberStatus {
    Idle,
    Approaching,
    Speaking,
    Leaving,
}

#[derive(Event, Clone)]
pub struct SubscriberMsg {
    pub name: String,
    pub msg: String,
}

#[derive(Bundle)]
pub struct SubscriberBundle {
    label: SubscriberLabel,
    sprite: SpriteBundle,
    texture_atlas: TextureAtlas,
    movement_type: GameEntityType,
    status: SubscriberStatus,
}

pub fn replace_subscriber_sprite(
    subscriber: Query<(Entity, &Transform, &TileTextureIndex), Added<SubscriberLabel>>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    for (subscriber_entity, subscriber_transform, tile_texture_index) in &subscriber {
        let texture_handle = asset_server.load("subscriber/Fish(32x32).png");
        let subscriber_texture_atlas =
            TextureAtlasLayout::from_grid(UVec2::new(32, 32), 16, 16, None, None);
        let subscriber_texture_atlas_handle = texture_atlases.add(subscriber_texture_atlas);

        let subscriber_texture_atlas = TextureAtlas {
            layout: subscriber_texture_atlas_handle.clone(),
            index: tile_texture_index.0 as usize,
        };

        let subscriber_sprite = SpriteBundle {
            sprite: Sprite::default(),
            texture: texture_handle.clone(),
            transform: *subscriber_transform,
            ..default()
        };

        commands.entity(subscriber_entity).remove::<Transform>();
        commands
            .entity(subscriber_entity)
            .insert((subscriber_sprite, subscriber_texture_atlas));
    }
}

/// Respawns Subscriber without rendering components
pub fn replace_subscriber_tile(
    mut tiles_query: Query<(Entity, &LayerNumber, &TilePos, &TileTextureIndex)>,
    map_info_query: Query<
        (&Transform, &TilemapGridSize, &TilemapSize, &TilemapType),
        Added<TilemapGridSize>,
    >,
    mut commands: Commands,
) {
    let map_information = map_info_query
        .iter()
        .find(|map_info| map_info.0.translation.z == DESIRED_SUBSCRIBER_LAYER_NUM as f32);

    if map_information.is_none() {
        return;
    }

    let (map_transform, grid_size, map_size, map_type) =
        map_information.expect("replace_subscriber_tile: Map information should exist by now.");

    for (subscriber_entity, layer_number, tile_pos, tile_texture_index) in &mut tiles_query {
        if layer_number.0 != SUBSCRIBER_LAYER_NUM {
            continue;
        }

        let map_info = TiledMapInformation::new(grid_size, map_size, map_type, map_transform);
        let tile_transform = to_bevy_transform(tile_pos, map_info);

        commands.entity(subscriber_entity).despawn_recursive();
        commands.spawn((
            (
                SubscriberLabel,
                GameEntityType::Swim,
                tile_transform,
                SubscriberStatus::Idle,
                *tile_texture_index,
            ),
            *tile_pos,
        ));
    }
}

pub fn trigger_swimming_to_streamer(
    mut subscriber_msg: EventWriter<SubscriberMsg>,
    pressed_key: Res<ButtonInput<KeyCode>>,
) {
    if !pressed_key.pressed(KeyCode::KeyU) {
        return;
    }

    let chat_msg = SubscriberMsg {
        name: String::from("Fishu"),
        msg: String::from("'ello Caveman!"),
    };

    subscriber_msg.send(chat_msg);
}

/// Returns a Path consisting of nodes only contained in
/// the Graph found in the Node Edges.
fn include_nodes_only_from(path_to_streamer: Path, water_graph_edges: &NodeEdges) -> Path {
    let mut stripped_path = Path(VecDeque::new());

    for node in path_to_streamer.0.iter() {
        if water_graph_edges.0[*node].is_empty() {
            break;
        }

        stripped_path.0.push_back(*node);
    }

    stripped_path
}

pub fn swim_to_streamer_to_speak(
    mut subscriber_msg: EventReader<SubscriberMsg>,
    mut subscriber: Query<
        (Entity, &TilePos, &mut Path, &mut SubscriberStatus),
        (With<SubscriberLabel>, Without<SubscriberMsg>),
    >,
    water_graph: Query<(&NodeEdges, &GraphType)>,
    streamer: Query<&TilePos, With<StreamerLabel>>,
    map_info: Query<&TilemapSize>,
    mut commands: Commands,
) {
    if water_graph.is_empty() || streamer.is_empty() || map_info.is_empty() {
        return;
    }

    // Why do we care about the air graph for someone swimming?
    // Because anyone who can fly has total coverage of the whole
    // map, meaning this would be a great reference for an initial
    // path to the Streamer before stripping out the tiles that a
    // Subscriber cannot traverse.
    let air_graph_edges = water_graph
        .iter()
        .find(|graph_elements| graph_elements.1 == &GraphType::Air)
        .map(|graph_elements| graph_elements.0)
        .expect("swim_to_streamer_to_speak: There should only be one air graph for reference.");
    let water_graph_edges = water_graph
        .iter()
        .find(|graph_elements| graph_elements.1 == &GraphType::Water)
        .map(|graph_elements| graph_elements.0)
        .expect("swim_to_streamer_to_speak: There should only be one water graph.");
    let streamer_tilepos = streamer
        .get_single()
        .expect("swim_to_streamer_to_speak: There should only be one streamer.");
    let map_size = map_info
        .iter()
        .next()
        .expect("swim_to_streamer_to_speak: There should be only one map.");
    for (subscriber_entity, subscriber_tilepos, mut subscriber_path, mut subscriber_status) in
        &mut subscriber
    {
        if subscriber_msg.is_empty() || *subscriber_status != SubscriberStatus::Idle {
            break;
        }

        if let Some(path) =
            air_graph_edges.shortest_path(*subscriber_tilepos, *streamer_tilepos, map_size.x)
        {
            let path_to_shore = include_nodes_only_from(path, water_graph_edges);
            *subscriber_path = path_to_shore;

            commands
                .entity(subscriber_entity)
                .insert(subscriber_msg.read().next().unwrap().clone());

            *subscriber_status = SubscriberStatus::Approaching;
        }
    }
}

pub fn speak_to_streamer_from_subscriber(
    mut subscriber_query: Query<
        (
            Entity,
            &SubscriberMsg,
            &Path,
            &Target,
            &mut SubscriberStatus,
            &GameEntityType,
        ),
        Without<WaitTimer>,
    >,
    mut chat_msg_requester: EventWriter<Msg>,
    mut commands: Commands,
) {
    for (
        subscriber_entity,
        subscriber_msg,
        subscriber_path,
        subscriber_target,
        mut subscriber_status,
        &subscriber_type,
    ) in &mut subscriber_query
    {
        if !subscriber_path.0.is_empty()
            || subscriber_target.is_some()
            || *subscriber_status != SubscriberStatus::Approaching
        {
            continue;
        }

        commands
            .entity(subscriber_entity)
            .insert(WaitTimer(Timer::from_seconds(60.0, TimerMode::Once)));

        *subscriber_status = SubscriberStatus::Speaking;
        chat_msg_requester.send(Msg::new(
            subscriber_msg.name.clone(),
            subscriber_msg.msg.clone(),
            subscriber_type,
        ));
    }
}

pub fn leave_from_streamer_from_subscriber(
    time: Res<Time>,
    mut subscriber: Query<(
        Entity,
        &mut WaitTimer,
        &mut Path,
        &StartingPoint,
        &SpawnPoint,
        &mut SubscriberStatus,
    )>,
    water_graph_info: Query<(&NodeEdges, &GraphType)>,
    map_info: Query<&TilemapSize>,
    mut commands: Commands,
) {
    if subscriber.is_empty() || water_graph_info.is_empty() || map_info.is_empty() {
        return;
    }

    let map_size = map_info
        .iter()
        .last()
        .expect("leave_from_streamer: Map should be spawned by now.");

    let water_graph_edges = water_graph_info
        .iter()
        .filter(|graph_info| *graph_info.1 == GraphType::Water)
        .map(|graph_info| graph_info.0)
        .next()
        .expect("leave_from_streamer: Exactly one water graph should exist by now.");

    for (
        subscriber_entity,
        mut subscriber_wait_time,
        mut subscriber_path,
        subscriber_start_pos,
        subscriber_spawn_pos,
        mut subscriber_status,
    ) in &mut subscriber
    {
        subscriber_wait_time.0.tick(time.delta());
        if !subscriber_wait_time.0.finished() {
            continue;
        }

        if let Some(path) = water_graph_edges.shortest_path(
            subscriber_start_pos.1,
            subscriber_spawn_pos.0,
            map_size.x,
        ) {
            *subscriber_path = path;

            commands
                .entity(subscriber_entity)
                .remove::<WaitTimer>()
                .remove::<SubscriberMsg>();

            *subscriber_status = SubscriberStatus::Leaving;
        }
    }
}

/// Sets the Subscriber's Status back to Idle
/// when reaching its starting position once
/// again after leaving.
pub fn return_subscriber_to_idle(
    mut subscriber: Query<(&Path, &Target, &mut SubscriberStatus), With<SubscriberLabel>>,
) {
    for (subscriber_path, subscriber_target, mut subscriber_status) in &mut subscriber {
        if *subscriber_status != SubscriberStatus::Leaving {
            continue;
        }

        if !subscriber_path.0.is_empty() {
            continue;
        }

        if subscriber_target.is_some() {
            continue;
        }

        *subscriber_status = SubscriberStatus::Idle;
    }
}

pub fn follow_streamer_while_approaching_for_subscriber(
    streamer_info: Query<(&StreamerState, &Path), Without<SubscriberStatus>>,
    mut subscriber_info: Query<(&SubscriberStatus, &TilePos, &mut Path), Without<StreamerState>>,
    air_graph_info: Query<(&NodeEdges, &GraphType)>,
    map_info: Query<&TilemapSize>,
) {
    if streamer_info.is_empty() || subscriber_info.is_empty() || map_info.is_empty() {
        return;
    }

    let map_size = map_info
        .iter()
        .last()
        .expect("follow_streamer_while_approaching: Map should be spawned by now.");

    let (streamer_status, streamer_path) = streamer_info
        .get_single()
        .expect("follow_streamer_while_approaching: Streamer should exist by now.");

    if *streamer_status != StreamerState::Moving {
        return;
    }

    if streamer_path.0.is_empty() {
        return;
    }

    let air_graph_edges = air_graph_info
        .iter()
        .filter(|graph_info| *graph_info.1 == GraphType::Air)
        .map(|graph_info| graph_info.0)
        .next()
        .expect("follow_streamer_while_approaching: Exactly one air graph should exist by now.");

    let water_graph_edges = air_graph_info
        .iter()
        .filter(|graph_info| *graph_info.1 == GraphType::Water)
        .map(|graph_info| graph_info.0)
        .next()
        .expect("follow_streamer_while_approaching: Exactly one water graph should exist by now.");

    for (subscriber_status, subscriber_pos, mut subscriber_path) in &mut subscriber_info {
        if *subscriber_status != SubscriberStatus::Approaching || subscriber_path.0.is_empty() {
            continue;
        }

        let streamer_destination = streamer_path
            .0
            .iter()
            .last()
            .expect("follow_streamer_while_approaching: Streamer Path should be populated.");
        let streamer_destination_tilepos = idx_to_tilepos(*streamer_destination, map_size.y);
        let current_subscriber_destination =
            idx_to_tilepos(*subscriber_path.0.iter().last().unwrap(), map_size.y);

        let path_to_shore = if let Some(path) =
            air_graph_edges.shortest_path(*subscriber_pos, streamer_destination_tilepos, map_size.x)
        {
            include_nodes_only_from(path, water_graph_edges)
        } else {
            Path(VecDeque::new())
        };

        let next_subscriber_destination_idx = path_to_shore
            .0
            .iter()
            .last()
            .expect("follow_streamer_while_approaching_from_subscriber: New path to Streamer does not exist.");

        if path_to_shore.0.is_empty() {
            continue;
        }

        let next_subscriber_destination =
            idx_to_tilepos(*next_subscriber_destination_idx, map_size.y);

        // We do not want to re-populate the path if the Subscriber is already
        // going to the desired destination.
        if current_subscriber_destination == next_subscriber_destination {
            continue;
        }

        // This accounts for the situation when the Subscriber
        // arrives before the Streamer does, and the Subscriber
        // is just waiting.
        if *subscriber_pos == next_subscriber_destination {
            continue;
        }

        *subscriber_path = path_to_shore;
    }
}
