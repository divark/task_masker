use bevy::prelude::*;
use bevy_ecs_tilemap::{
    prelude::{TilemapGridSize, TilemapSize, TilemapType},
    tiles::TilePos,
};

use crate::map::plugins::TilePosEvent;
use crate::map::tiled::{to_bevy_transform, TiledMapInformation};
use crate::map::{path_finding::*, tilemap::TileGridCoordinates};
use crate::ui::chatting::ChattingStatus;

use super::GameEntityType;

#[derive(Component)]
pub struct StreamerLabel;

#[derive(Component, PartialEq, Copy, Clone, Debug, Eq, Hash)]
pub enum StreamerState {
    Idle,
    Moving,
    Speaking,
    // TODO: Should Action be included?
}

/// This represents the Online presense of the
/// Streamer, unlike the StreamerState, which
/// represents the Streamer Entity's State in the
/// game.
#[derive(Event)]
pub enum OnlineStatus {
    Online,
    Away,
}

#[derive(Bundle)]
pub struct Streamer {
    label: StreamerLabel,
    sprites: SpriteBundle,
    movement_type: GameEntityType,
    status: StreamerState,
}

pub const STREAMER_LAYER_NUM: usize = 6;

pub fn spawn_player_sprite(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    streamer_query: Query<(Entity, &Transform), (With<StreamerLabel>, Without<TextureAtlas>)>,
) {
    if streamer_query.is_empty() {
        return;
    }

    let (streamer_entity, streamer_transform) = streamer_query
        .get_single()
        .expect("spawn_player: Could not find Streamer.");

    let texture_handle = asset_server.load("caveman/caveman-sheet.png");
    let texture_atlas = TextureAtlasLayout::from_grid(UVec2::new(16, 16), 4, 9, None, None);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    let streamer_texture_atlas = TextureAtlas {
        layout: texture_atlas_handle,
        index: 0,
    };

    let streamer_sprite = SpriteBundle {
        sprite: Sprite::default(),
        transform: *streamer_transform,
        texture: texture_handle,
        ..default()
    };

    commands.entity(streamer_entity).remove::<Transform>();
    commands
        .entity(streamer_entity)
        .insert((streamer_sprite, streamer_texture_atlas));
}

/// Spawns Player without any component related to rendering
pub fn spawn_player_tile(
    mut commands: Commands,
    map_information: Query<
        (&Transform, &TilemapType, &TilemapGridSize, &TilemapSize),
        Added<TilemapType>,
    >,
    streamer_query: Query<(), With<StreamerLabel>>,
) {
    if !streamer_query.is_empty() {
        return;
    }

    if map_information.is_empty() {
        return;
    }

    let (map_transform, map_type, grid_size, map_size) = map_information
        .iter()
        .nth(STREAMER_LAYER_NUM)
        .expect("Could not load map information. Is world loaded?");
    let map_info = TiledMapInformation::new(grid_size, map_size, map_type, map_transform);

    let streamer_bevy_tilepos = TilePos::new(39, 40);
    let streamer_transform = to_bevy_transform(&streamer_bevy_tilepos, map_info);

    commands.spawn((
        (
            StreamerLabel,
            GameEntityType::Walk,
            StreamerState::Idle,
            streamer_transform,
        ),
        streamer_bevy_tilepos,
    ));
}

pub fn move_streamer(
    mut streamer_entity: Query<
        (
            &mut Path,
            &StartingPoint,
            &Target,
            &mut DestinationQueue,
            &mut StreamerState,
        ),
        With<StreamerLabel>,
    >,
    ground_graph_query: Query<&UndirectedGraph>,
) {
    if streamer_entity.is_empty() || ground_graph_query.is_empty() {
        return;
    }

    let ground_graph = ground_graph_query
        .iter()
        .find(|graph| *graph.get_node_type() == GraphType::Ground)
        .expect("move_streamer: Could not find Ground-based UndirectedGraph.");

    let (
        mut streamer_path,
        streamer_tile_pos,
        streamer_target,
        mut streamer_destination_queue,
        mut streamer_status,
    ) = streamer_entity
        .get_single_mut()
        .expect("The streamer should be loaded.");

    if !streamer_path.is_empty() || streamer_target.is_some() {
        return;
    }

    if streamer_destination_queue.is_empty() {
        return;
    }

    let streamer_target = streamer_destination_queue.pop_front().expect(
        "move_streamer: Destination queue for streamer should have been filled with something.",
    );
    if let Some(found_path) =
        ground_graph.shortest_path(streamer_tile_pos.1.clone(), streamer_target)
    {
        *streamer_path = found_path;
        *streamer_status = StreamerState::Moving;
    }
}

/// Updates the Status of the Streamer to Idle when the Streamer
/// is no longer following some path.
pub fn make_streamer_idle_when_not_moving(
    mut streamer: Query<(&mut StreamerState, &Path, &Target)>,
) {
    if streamer.is_empty() {
        return;
    }

    let (mut streamer_status, streamer_path, streamer_target) = streamer.single_mut();

    if streamer_path.len() == 0 && streamer_target.is_none() {
        *streamer_status = StreamerState::Idle;
    }
}

pub fn queue_destination_for_streamer(
    mut destination_request_listener: EventReader<TilePosEvent>,
    mut streamer_entity: Query<&mut DestinationQueue, With<StreamerLabel>>,
) {
    if streamer_entity.is_empty() {
        return;
    }

    let mut streamer_destination_queue = streamer_entity.single_mut();
    for destination_info in &mut destination_request_listener.read() {
        streamer_destination_queue.push_back(destination_info.destination.clone());
    }
}

pub fn change_status_for_streamer(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut online_status_writer: EventWriter<OnlineStatus>,
) {
    let one_pressed = keyboard_input.just_pressed(KeyCode::Numpad1)
        || keyboard_input.just_pressed(KeyCode::Digit1);
    let two_pressed = keyboard_input.just_pressed(KeyCode::Numpad2)
        || keyboard_input.just_pressed(KeyCode::Digit2);

    if one_pressed {
        online_status_writer.send(OnlineStatus::Online);
    } else if two_pressed {
        online_status_writer.send(OnlineStatus::Away);
    }
}

/// Requests the Streamer to move to a point of interest
/// based on a change in Online Status.
pub fn move_streamer_on_status_change(
    mut online_status_listener: EventReader<OnlineStatus>,
    mut destination_request_writer: EventWriter<TilePosEvent>,
) {
    for new_online_status in &mut online_status_listener.read() {
        let new_destination = match new_online_status {
            OnlineStatus::Online => TileGridCoordinates::new(41, 49),
            OnlineStatus::Away => TileGridCoordinates::new(39, 40),
        };

        destination_request_writer.send(TilePosEvent::new(new_destination));
    }
}

pub fn update_status_when_speaking(
    chatting_query: Query<&ChattingStatus, Changed<ChattingStatus>>,
    mut streamer_query: Query<&mut StreamerState>,
) {
    if streamer_query.is_empty() {
        return;
    }

    let mut streamer_status = streamer_query
        .get_single_mut()
        .expect("update_status_when_speaking: Streamer's status should exist by now.");
    for chatting_status in &chatting_query {
        if *chatting_status != ChattingStatus::Speaking(GameEntityType::Walk) {
            continue;
        }

        *streamer_status = StreamerState::Speaking;
    }
}
