use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use std::collections::VecDeque;

use crate::entities::streamer::{StreamerLabel, StreamerStatus};
use crate::map::path_finding::*;
use crate::map::tiled::{
    flip_y_axis_for_tile_pos, to_bevy_transform, LayerNumber, TiledMapInformation,
};
use crate::ui::chatting::Msg;

use super::MovementType;

pub const CHATTER_LAYER_NUM: usize = 19;
pub const DIST_AWAY_FROM_STREAMER: usize = 2;

#[derive(Component)]
pub struct ChatterLabel;

#[derive(Component, PartialEq)]
pub enum ChatterStatus {
    Idle,
    Approaching,
    Speaking,
    Leaving,
}

#[derive(Component)]
pub struct WaitTimer(Timer);

#[derive(Component, Event, Clone)]
pub struct ChatMsg {
    pub name: String,
    pub msg: String,
}

#[derive(Bundle)]
pub struct ChatterBundle {
    label: ChatterLabel,
    sprite: SpriteSheetBundle,
    movement_type: MovementType,
    status: ChatterStatus,
}

pub fn replace_chatter_sprite(
    chatter: Query<(Entity, &Transform, &TileTextureIndex), Added<ChatterLabel>>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    for (chatter_entity, chatter_transform, tile_texture_index) in &chatter {
        let texture_handle = asset_server.load("chatters/animation.png");
        let chatter_texture_atlas =
            TextureAtlasLayout::from_grid(Vec2::new(16.0, 16.0), 8, 2, None, None);
        let chatter_texture_atlas_handle = texture_atlases.add(chatter_texture_atlas);

        let chatter_sprite = SpriteSheetBundle {
            sprite: Sprite::default(),
            atlas: TextureAtlas {
                layout: chatter_texture_atlas_handle.clone(),
                index: tile_texture_index.0 as usize,
            },
            texture: texture_handle.clone(),
            transform: *chatter_transform,
            ..default()
        };

        commands.entity(chatter_entity).remove::<Transform>();
        commands.entity(chatter_entity).insert(chatter_sprite);
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
                MovementType::Fly,
                ChatterStatus::Idle,
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
        msg: String::from("Hello Caveman!"),
    };

    chatter_msg.send(chat_msg);
}

pub fn fly_to_streamer_to_speak(
    mut chatter_msg: EventReader<ChatMsg>,
    mut chatter: Query<
        (Entity, &TilePos, &mut Path, &mut ChatterStatus),
        (With<ChatterLabel>, Without<ChatMsg>),
    >,
    air_graph: Query<(&NodeEdges, &GraphType)>,
    streamer: Query<&TilePos, With<StreamerLabel>>,
    map_info: Query<&TilemapSize>,
    mut commands: Commands,
) {
    if air_graph.is_empty() || streamer.is_empty() || map_info.is_empty() {
        return;
    }

    let air_graph_edges = air_graph
        .iter()
        .find(|graph_elements| graph_elements.1 == &GraphType::Air)
        .expect("fly_to_streamer_to_speak: There should only be one air graph.");
    let streamer_tilepos = streamer
        .get_single()
        .expect("fly_to_streamer_to_speak: There should only be one streamer.");
    let map_size = map_info
        .iter()
        .next()
        .expect("fly_to_streamer_to_speak: There should be only one map.");
    for (chatter_entity, chatter_tilepos, mut chatter_path, mut chatter_status) in &mut chatter {
        if chatter_msg.is_empty() || *chatter_status != ChatterStatus::Idle {
            break;
        }

        let mut path_to_streamer = get_path(
            chatter_tilepos,
            streamer_tilepos,
            map_size,
            air_graph_edges.0,
        );

        // The chatter should not be directly on top of the
        // streamer, so we provide some distance by adjusting
        // the path to not go straight to the streamer.
        for _i in 0..DIST_AWAY_FROM_STREAMER {
            path_to_streamer.pop_back();
        }

        *chatter_path = path_to_streamer;

        commands
            .entity(chatter_entity)
            .insert(chatter_msg.read().next().unwrap().clone());

        *chatter_status = ChatterStatus::Approaching;
    }
}

pub fn speak_to_streamer_from_chatter(
    mut chatter_query: Query<
        (Entity, &ChatMsg, &Path, &mut ChatterStatus, &MovementType),
        Without<WaitTimer>,
    >,
    mut chat_msg_requester: EventWriter<Msg>,
    mut commands: Commands,
) {
    for (chatter_entity, chatter_msg, chatter_path, mut chatter_status, &chatter_type) in
        &mut chatter_query
    {
        if !chatter_path.0.is_empty() || *chatter_status != ChatterStatus::Approaching {
            continue;
        }

        commands
            .entity(chatter_entity)
            .insert(WaitTimer(Timer::from_seconds(10.0, TimerMode::Once)));

        *chatter_status = ChatterStatus::Speaking;
        chat_msg_requester.send(Msg {
            speaker_name: chatter_msg.name.clone(),
            speaker_role: chatter_type,
            msg: chatter_msg.msg.clone(),
        });
    }
}

pub fn leave_from_streamer_from_chatter(
    time: Res<Time>,
    mut chatter: Query<(
        Entity,
        &mut WaitTimer,
        &mut Path,
        &StartingPoint,
        &SpawnPoint,
        &mut ChatterStatus,
    )>,
    air_graph_info: Query<(&NodeEdges, &GraphType)>,
    map_info: Query<&TilemapSize>,
    mut commands: Commands,
) {
    if chatter.is_empty() || air_graph_info.is_empty() || map_info.is_empty() {
        return;
    }

    let map_size = map_info
        .iter()
        .last()
        .expect("leave_from_streamer: Map should be spawned by now.");

    let air_graph_edges = air_graph_info
        .iter()
        .filter(|graph_info| *graph_info.1 == GraphType::Air)
        .map(|graph_info| graph_info.0)
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

        *chatter_path = get_path(
            &chatter_start_pos.1,
            &chatter_spawn_pos.0,
            map_size,
            air_graph_edges,
        );

        commands
            .entity(chatter_entity)
            .remove::<WaitTimer>()
            .remove::<ChatMsg>();

        *chatter_status = ChatterStatus::Leaving;
    }
}

/// Sets the Chatter's Status back to Idle
/// when reaching its starting position once
/// again after leaving.
pub fn return_chatter_to_idle(
    mut chatter: Query<(&Path, &mut ChatterStatus), (Changed<Path>, With<ChatterLabel>)>,
) {
    for (chatter_path, mut chatter_status) in &mut chatter {
        if *chatter_status != ChatterStatus::Leaving {
            continue;
        }

        if !chatter_path.0.is_empty() {
            continue;
        }

        *chatter_status = ChatterStatus::Idle;
    }
}

pub fn follow_streamer_while_speaking(
    streamer_info: Query<(&StreamerStatus, &Path), Changed<StreamerStatus>>,
    mut chatter_info: Query<(&ChatterStatus, &mut Path), Without<StreamerStatus>>,
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

    if *streamer_status != StreamerStatus::Moving {
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
    streamer_info: Query<(&StreamerStatus, &Path), Without<ChatterStatus>>,
    mut chatter_info: Query<(&ChatterStatus, &TilePos, &mut Path), Without<StreamerStatus>>,
    air_graph_info: Query<(&NodeEdges, &GraphType)>,
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

    if *streamer_status != StreamerStatus::Moving {
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

        *chatter_path = get_path(
            chatter_pos,
            &chatter_destination_distanced,
            map_size,
            air_graph_edges,
        );
    }
}
