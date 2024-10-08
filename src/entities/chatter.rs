use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use std::collections::VecDeque;

use crate::entities::streamer::{StreamerLabel, StreamerState};
use crate::entities::WaitToLeaveTimer;
use crate::map::path_finding::*;
use crate::map::tiled::{to_bevy_transform, LayerNumber, TiledMapInformation};
use crate::ui::chatting::{Msg, TypingMsg};

use super::GameEntityType;

pub const CHATTER_LAYER_NUM: usize = 19;
pub const DIST_AWAY_FROM_STREAMER: usize = 2;

#[derive(Component)]
pub struct ChatterLabel;

#[derive(Component, Debug, PartialEq)]
pub enum ChatterStatus {
    Idle,
    Approaching,
    Speaking,
    Leaving,
}

#[derive(Event, Clone)]
pub struct ChatMsg {
    pub name: String,
    pub msg: String,
}

#[derive(Component, Deref, DerefMut)]
pub struct ChatMessageQueue(VecDeque<ChatMsg>);

#[derive(Bundle)]
pub struct ChatterBundle {
    label: ChatterLabel,
    sprite: SpriteBundle,
    texture_atlas: TextureAtlas,
    movement_type: GameEntityType,
    status: ChatterStatus,
}

pub fn replace_chatter_sprite(
    chatter: Query<(Entity, &Transform, &TileTextureIndex), Added<ChatterLabel>>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    for (chatter_entity, chatter_transform, tile_texture_index) in &chatter {
        let texture_handle = asset_server.load("chatter/BirdSprite (16x16).png");
        let chatter_texture_atlas =
            TextureAtlasLayout::from_grid(UVec2::new(16, 16), 8, 3, None, None);
        let chatter_texture_atlas_handle = texture_atlases.add(chatter_texture_atlas);

        let chatter_texture_atlas = TextureAtlas {
            layout: chatter_texture_atlas_handle.clone(),
            index: tile_texture_index.0 as usize,
        };

        let chatter_sprite = SpriteBundle {
            sprite: Sprite::default(),
            texture: texture_handle.clone(),
            transform: *chatter_transform,
            ..default()
        };

        commands.entity(chatter_entity).remove::<Transform>();
        commands
            .entity(chatter_entity)
            .insert((chatter_sprite, chatter_texture_atlas));
    }
}

/// Respawns Chatter without rendering components
pub fn replace_chatter_tile(
    tiles_query: Query<(Entity, &LayerNumber, &TilePos, &TileTextureIndex)>,
    map_info_query: Query<
        (&Transform, &TilemapGridSize, &TilemapSize, &TilemapType),
        Added<TilemapGridSize>,
    >,
    mut commands: Commands,
) {
    let map_information = map_info_query
        .iter()
        .find(|map_info| map_info.0.translation.z == CHATTER_LAYER_NUM as f32);

    if map_information.is_none() {
        return;
    }

    let (map_transform, grid_size, map_size, map_type) =
        map_information.expect("mock_replace_chatter: Map information should exist by now.");

    for (chatter_entity, layer_number, tile_pos, tile_texture_index) in &tiles_query {
        if layer_number.0 != CHATTER_LAYER_NUM {
            continue;
        }

        let map_info = TiledMapInformation::new(grid_size, map_size, map_type, map_transform);
        let tile_transform = to_bevy_transform(tile_pos, map_info);

        commands.entity(chatter_entity).despawn_recursive();
        commands.spawn((
            (
                ChatterLabel,
                tile_transform,
                GameEntityType::Fly,
                ChatterStatus::Idle,
                ChatMessageQueue(VecDeque::new()),
                *tile_texture_index,
            ),
            *tile_pos,
        ));
    }
}

pub fn trigger_flying_to_streamer(
    mut chatter_msg: EventWriter<ChatMsg>,
    pressed_key: Res<ButtonInput<KeyCode>>,
) {
    if !pressed_key.pressed(KeyCode::KeyG) {
        return;
    }

    let chat_msg = ChatMsg {
        name: String::from("Bob"),
        msg: String::from("So, if you're learning a subject of math for the first time, it's helpful to actually learn about the concepts behind it before going into the course, since you're otherwise being overloaded with a bunch of terminology. Doing it this way, it's important to do so with the angle of finding how it's important to your work, using analogies and metaphors to make the knowledge personal")
    };

    chatter_msg.send(chat_msg);
}

/// Adds a recently broadcasted Chat Message into a Chatter's
/// Chat Message Queue.
pub fn add_chat_msg_to_queue(
    mut received_chat_msgs: EventReader<ChatMsg>,
    mut chatter_queues: Query<&mut ChatMessageQueue>,
) {
    if chatter_queues.is_empty() {
        return;
    }

    let mut chatter_queue = chatter_queues.single_mut();
    for received_chat_msg in received_chat_msgs.read() {
        chatter_queue.push_back(received_chat_msg.clone());
    }
}

pub fn fly_to_streamer_to_speak(
    mut chatter: Query<
        (&TilePos, &mut Path, &mut ChatterStatus, &ChatMessageQueue),
        With<ChatterLabel>,
    >,
    air_graph: Query<&UndirectedGraph>,
    streamer: Query<&TilePos, With<StreamerLabel>>,
) {
    if air_graph.is_empty() || streamer.is_empty() {
        return;
    }

    let air_graph = air_graph
        .iter()
        .find(|graph| *graph.get_node_type() == GraphType::Air)
        .expect("fly_to_streamer_to_speak: There should only be one air graph.");
    let streamer_tilepos = streamer
        .get_single()
        .expect("fly_to_streamer_to_speak: There should only be one streamer.");
    for (chatter_tilepos, mut chatter_path, mut chatter_status, chatter_message_queue) in
        &mut chatter
    {
        if chatter_message_queue.is_empty() || *chatter_status != ChatterStatus::Idle {
            continue;
        }

        if let Some(mut path) = air_graph.shortest_path(*chatter_tilepos, *streamer_tilepos) {
            // The chatter should not be directly on top of the
            // streamer, so we provide some distance by adjusting
            // the path to not go straight to the streamer.
            for _i in 0..DIST_AWAY_FROM_STREAMER {
                path.pop_back();
            }

            *chatter_path = path;
            *chatter_status = ChatterStatus::Approaching;
        }
    }
}

pub fn speak_to_streamer_from_chatter(
    mut chatter_query: Query<(
        &mut ChatMessageQueue,
        &Path,
        &Target,
        &mut ChatterStatus,
        &GameEntityType,
    )>,
    mut chat_msg_requester: EventWriter<Msg>,
) {
    for (
        mut chatter_message_queue,
        chatter_path,
        chatter_target,
        mut chatter_status,
        &chatter_type,
    ) in &mut chatter_query
    {
        if !chatter_path.0.is_empty()
            || chatter_target.is_some()
            || chatter_message_queue.is_empty()
            || *chatter_status != ChatterStatus::Approaching
        {
            continue;
        }

        let recent_chat_msg = chatter_message_queue.pop_front().unwrap();
        *chatter_status = ChatterStatus::Speaking;
        chat_msg_requester.send(Msg::new(
            recent_chat_msg.name,
            recent_chat_msg.msg,
            chatter_type,
        ));
    }
}

/// Starts to wait to leave when the Chatter is finished speaking.
pub fn chatter_waits_to_leave_from_streamer(
    typed_messages: Query<&TypingMsg>,
    mut chatters: Query<(Entity, &mut ChatterStatus, &ChatMessageQueue), Without<WaitToLeaveTimer>>,
    mut commands: Commands,
) {
    if typed_messages.is_empty() || chatters.is_empty() {
        return;
    }

    let (chatter_entity, mut chatter_status, chatter_message_queue) = chatters.single_mut();
    if *chatter_status != ChatterStatus::Speaking {
        return;
    }

    let typing_msg = typed_messages.single();
    if !typing_msg.at_end() {
        return;
    }

    let next_message = chatter_message_queue.front();
    if next_message.is_none() || next_message.unwrap().name != typing_msg.speaker_name() {
        commands
            .entity(chatter_entity)
            .insert(WaitToLeaveTimer(Timer::from_seconds(10.0, TimerMode::Once)));
        return;
    }

    *chatter_status = ChatterStatus::Idle;
}

pub fn leave_from_streamer_from_chatter(
    time: Res<Time>,
    mut chatter: Query<(
        Entity,
        &mut WaitToLeaveTimer,
        &mut Path,
        &StartingPoint,
        &SpawnPoint,
        &mut ChatterStatus,
    )>,
    air_graph_info: Query<&UndirectedGraph>,
    mut commands: Commands,
) {
    if chatter.is_empty() || air_graph_info.is_empty() {
        return;
    }

    let air_graph = air_graph_info
        .iter()
        .filter(|graph| *graph.get_node_type() == GraphType::Air)
        .next()
        .expect("leave_from_streamer: Exactly one air graph should exist by now.");

    for (
        chatter_entity,
        mut chatter_wait_time,
        mut chatter_path,
        chatter_start_pos,
        chatter_spawn_pos,
        mut chatter_status,
    ) in &mut chatter
    {
        chatter_wait_time.0.tick(time.delta());
        if !chatter_wait_time.0.finished() {
            continue;
        }

        if let Some(path) = air_graph.shortest_path(chatter_start_pos.1, chatter_spawn_pos.0) {
            *chatter_path = path;
            commands.entity(chatter_entity).remove::<WaitToLeaveTimer>();

            *chatter_status = ChatterStatus::Leaving;
        }
    }
}

/// Sets the Chatter's Status back to Idle
/// when reaching its starting position once
/// again after leaving.
pub fn return_chatter_to_idle(
    mut chatter: Query<(&Path, &Target, &mut ChatterStatus), With<ChatterLabel>>,
) {
    for (chatter_path, chatter_target, mut chatter_status) in &mut chatter {
        if *chatter_status != ChatterStatus::Leaving {
            continue;
        }

        if !chatter_path.0.is_empty() {
            continue;
        }

        if chatter_target.is_some() {
            continue;
        }

        *chatter_status = ChatterStatus::Idle;
    }
}

pub fn follow_streamer_while_speaking(
    streamer_info: Query<(&StreamerState, &Path), Changed<StreamerState>>,
    mut chatter_info: Query<(&ChatterStatus, &mut Path), Without<StreamerState>>,
    map_info: Query<&TilemapSize>,
) {
    if streamer_info.is_empty() || chatter_info.is_empty() || map_info.is_empty() {
        return;
    }

    let map_size = map_info
        .iter()
        .last()
        .expect("follow_streamer_while_speaking: Map should be spawned by now.");

    let (streamer_status, streamer_path) = streamer_info
        .get_single()
        .expect("follow_streamer_while_speaking: Streamer should exist by now.");

    if *streamer_status != StreamerState::Moving {
        return;
    }

    for (chatter_status, mut chatter_path) in &mut chatter_info {
        if *chatter_status != ChatterStatus::Speaking || !chatter_path.is_empty() {
            continue;
        }

        chatter_path.0 = streamer_path
            .0
            .iter()
            .map(|target| idx_to_tilepos(*target, map_size.y))
            .map(|tilepos| {
                TilePos::new(
                    tilepos.x + DIST_AWAY_FROM_STREAMER as u32,
                    tilepos.y + DIST_AWAY_FROM_STREAMER as u32,
                )
            })
            .map(|tilepos_adjusted| {
                tilepos_to_idx(tilepos_adjusted.x, tilepos_adjusted.y, map_size.y)
            })
            .collect::<VecDeque<usize>>();
    }
}

pub fn follow_streamer_while_approaching_for_chatter(
    streamer_info: Query<(&StreamerState, &Path), Without<ChatterStatus>>,
    mut chatter_info: Query<(&ChatterStatus, &TilePos, &mut Path), Without<StreamerState>>,
    air_graph_info: Query<&UndirectedGraph>,
    map_info: Query<&TilemapSize>,
) {
    if streamer_info.is_empty() || chatter_info.is_empty() || map_info.is_empty() {
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

    let air_graph = air_graph_info
        .iter()
        .filter(|graph| *graph.get_node_type() == GraphType::Air)
        .next()
        .expect("follow_streamer_while_approaching: Exactly one air graph should exist by now.");

    for (chatter_status, chatter_pos, mut chatter_path) in &mut chatter_info {
        if *chatter_status != ChatterStatus::Approaching || chatter_path.0.is_empty() {
            continue;
        }

        let streamer_destination = streamer_path
            .0
            .iter()
            .last()
            .expect("follow_streamer_while_approaching: Streamer Path should be populated.");
        let streamer_destination_tilepos = idx_to_tilepos(*streamer_destination, map_size.y);
        let chatter_destination_distanced = TilePos::new(
            streamer_destination_tilepos.x + DIST_AWAY_FROM_STREAMER as u32,
            streamer_destination_tilepos.y + DIST_AWAY_FROM_STREAMER as u32,
        );

        let current_chatter_destination =
            idx_to_tilepos(*chatter_path.0.iter().last().unwrap(), map_size.y);

        // We do not want to re-populate the path if the Chatter is already
        // going to the desired destination.
        if current_chatter_destination == chatter_destination_distanced {
            continue;
        }

        // This accounts for the situation when the Chatter
        // arrives before the Streamer does, and the Chatter
        // is just waiting.
        if *chatter_pos == chatter_destination_distanced {
            continue;
        }

        if let Some(path) = air_graph.shortest_path(*chatter_pos, chatter_destination_distanced) {
            *chatter_path = path;
        }
    }
}
