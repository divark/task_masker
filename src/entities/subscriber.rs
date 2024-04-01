use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use std::collections::VecDeque;

use crate::entities::streamer::StreamerLabel;
use crate::map::path_finding::*;
use crate::map::tiled::{tiled_to_tile_pos, to_bevy_transform, LayerNumber, TiledMapInformation};
use crate::ui::chatting::Msg;

use super::MovementType;

pub const SUBSCRIBER_LAYER_NUM: usize = 19;

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

pub fn trigger_swimming_to_streamer(
    mut subscriber_msg: EventWriter<SubscriberMsg>,
    pressed_key: Res<Input<KeyCode>>,
) {
    if !pressed_key.pressed(KeyCode::U) {
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
            continue;
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

        let path_to_streamer = get_path(
            subscriber_tilepos,
            streamer_tilepos,
            map_size,
            air_graph_edges,
        );

        let path_to_shore = include_nodes_only_from(path_to_streamer, water_graph_edges);

        *subscriber_path = path_to_shore;

        commands
            .entity(subscriber_entity)
            .insert(subscriber_msg.read().next().unwrap().clone());

        *subscriber_status = SubscriberStatus::Approaching;
    }
}

pub fn speak_to_streamer_from_subscriber(
    mut subscriber_query: Query<
        (
            Entity,
            &SubscriberMsg,
            &Path,
            &mut SubscriberStatus,
            &MovementType,
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
        mut subscriber_status,
        &subscriber_type,
    ) in &mut subscriber_query
    {
        if !subscriber_path.0.is_empty() || *subscriber_status != SubscriberStatus::Approaching {
            continue;
        }

        commands
            .entity(subscriber_entity)
            .insert(WaitTimer(Timer::from_seconds(60.0, TimerMode::Once)));

        *subscriber_status = SubscriberStatus::Speaking;
        chat_msg_requester.send(Msg {
            speaker_name: subscriber_msg.name.clone(),
            speaker_role: subscriber_type,
            msg: subscriber_msg.msg.clone(),
        });
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

        *subscriber_path = get_path(
            &subscriber_start_pos.1,
            &subscriber_spawn_pos.0,
            map_size,
            water_graph_edges,
        );

        commands
            .entity(subscriber_entity)
            .remove::<WaitTimer>()
            .remove::<SubscriberMsg>();

        *subscriber_status = SubscriberStatus::Leaving;
    }
}

/// Sets the Subscriber's Status back to Idle
/// when reaching its starting position once
/// again after leaving.
pub fn return_subscriber_to_idle(
    mut subscriber: Query<(&Path, &mut SubscriberStatus), (Changed<Path>, With<SubscriberLabel>)>,
) {
    for (subscriber_path, mut subscriber_status) in &mut subscriber {
        if *subscriber_status != SubscriberStatus::Leaving {
            continue;
        }

        if !subscriber_path.0.is_empty() {
            continue;
        }

        *subscriber_status = SubscriberStatus::Idle;
    }
}
