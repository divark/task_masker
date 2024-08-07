use bevy::prelude::*;
use bevy_ecs_tilemap::{
    prelude::{TilemapGridSize, TilemapSize, TilemapType},
    tiles::TilePos,
};

use crate::map::path_finding::*;
use crate::map::plugins::TilePosEvent;
use crate::map::tiled::{convert_tiled_to_bevy_pos, to_bevy_transform, TiledMapInformation};
use crate::ui::chatting::{ChattingStatus, Msg};

use super::MovementType;

#[derive(Component)]
pub struct StreamerLabel;

#[derive(Component, PartialEq, Copy, Clone, Debug, Eq, Hash)]
pub enum StreamerStatus {
    Idle,
    Moving,
    Speaking,
    // TODO: Should Action be included?
}

#[derive(Bundle)]
pub struct Streamer {
    label: StreamerLabel,
    sprites: SpriteBundle,
    movement_type: MovementType,
    status: StreamerStatus,
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

    let streamer_tiled_tilepos = TilePos { x: 42, y: 59 };
    let streamer_bevy_tilepos = convert_tiled_to_bevy_pos(streamer_tiled_tilepos, map_size.x);
    let streamer_transform = to_bevy_transform(&streamer_bevy_tilepos, map_info);

    commands.spawn((
        (
            StreamerLabel,
            MovementType::Walk,
            StreamerStatus::Idle,
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
            &mut StreamerStatus,
        ),
        With<StreamerLabel>,
    >,
    ground_graph_query: Query<(&NodeEdges, &GraphType)>,
    map_information: Query<(&TilemapSize, &Transform)>,
) {
    if streamer_entity.is_empty() {
        return;
    }

    let ground_graph = ground_graph_query
        .iter()
        .find(|graph_elements| graph_elements.1 == &GraphType::Ground);
    if ground_graph.is_none() {
        return;
    }

    let edges = ground_graph.expect("Ground graph should be loaded.").0;
    // Each Tile Layer has its own World and Grid size should someone decide
    // to change tilesets for the layer. However, I will not do that, so
    // both the world size and grid size should be the same.
    let map_size = map_information
        .iter()
        .map(|sizes| sizes.0)
        .max_by(|&x, &y| {
            let x_world_area = x.x * x.y;
            let y_world_area = y.x * y.y;

            x_world_area.cmp(&y_world_area)
        })
        .expect("Could not find largest world size. Is the map loaded?");

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
    if let Some(found_path) = edges.shortest_path(streamer_tile_pos.1, streamer_target, map_size.x)
    {
        *streamer_path = found_path;
        *streamer_status = StreamerStatus::Moving;
    }
}

/// Updates the Status of the Streamer to Idle when the Streamer
/// is no longer following some path.
pub fn make_streamer_idle_when_not_moving(
    mut streamer: Query<(&mut StreamerStatus, &Path, &Target)>,
) {
    if streamer.is_empty() {
        return;
    }

    let (mut streamer_status, streamer_path, streamer_target) = streamer.single_mut();

    if streamer_path.len() == 0 && streamer_target.is_none() {
        *streamer_status = StreamerStatus::Idle;
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
        streamer_destination_queue.push_back(destination_info.destination);
    }
}

pub fn move_streamer_on_spacebar(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut destination_request_writer: EventWriter<TilePosEvent>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        destination_request_writer.send(TilePosEvent::new(TilePos { x: 64, y: 47 }));
    }
}

pub fn test_streamer_msg(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut msg_writer: EventWriter<Msg>,
) {
    if !keyboard_input.just_pressed(KeyCode::KeyQ) {
        return;
    }

    let streamer_msg = Msg::new(
        "Caveman".to_string(),
        "This is a test message to see if this works and types as expected.".to_string(),
        MovementType::Walk,
    );

    msg_writer.send(streamer_msg);
}

pub fn update_status_when_speaking(
    chatting_query: Query<&ChattingStatus, Changed<ChattingStatus>>,
    mut streamer_query: Query<&mut StreamerStatus>,
) {
    if streamer_query.is_empty() {
        return;
    }

    let mut streamer_status = streamer_query
        .get_single_mut()
        .expect("update_status_when_speaking: Streamer's status should exist by now.");
    for chatting_status in &chatting_query {
        if *chatting_status != ChattingStatus::Speaking(MovementType::Walk) {
            continue;
        }

        *streamer_status = StreamerStatus::Speaking;
    }
}
