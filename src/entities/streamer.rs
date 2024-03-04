use bevy::prelude::*;
use bevy_ecs_tilemap::{
    prelude::{TilemapGridSize, TilemapSize, TilemapType},
    tiles::TilePos,
};

use crate::map::path_finding::*;
use crate::map::plugins::TilePosEvent;
use crate::map::tiled::{tiled_to_bevy_transform, TiledMapInformation};
use crate::ui::chatting::Msg;

use super::MovementType;

#[derive(Component)]
pub struct StreamerLabel;

#[derive(Component)]
pub enum StreamerStatus {
    Idle,
    Moving,
    Speaking,
    // TODO: Should Action be included?
}

#[derive(Bundle)]
pub struct Streamer {
    label: StreamerLabel,
    sprites: SpriteSheetBundle,
    movement_type: MovementType,
}

pub fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    map_information: Query<
        (&Transform, &TilemapType, &TilemapGridSize, &TilemapSize),
        Added<TilemapType>,
    >,
    streamer_query: Query<(), With<StreamerLabel>>,
) {
    if !streamer_query.is_empty() {
        return;
    }

    let texture_handle = asset_server.load("caveman/caveman-sheet.png");
    let texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::new(16.0, 16.0), 4, 9, None, None);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    if map_information.is_empty() {
        return;
    }

    let (map_transform, map_type, grid_size, map_size) = map_information
        .iter()
        .nth(6)
        .expect("Could not load map information. Is world loaded?");
    let map_info = TiledMapInformation::new(grid_size, map_size, map_type, map_transform);

    let streamer_location = TilePos { x: 38, y: 59 }; //{ x: 42, y: 59 };
    let streamer_transform = tiled_to_bevy_transform(&streamer_location, map_info);

    commands.spawn((
        Streamer {
            label: StreamerLabel,
            sprites: SpriteSheetBundle {
                texture_atlas: texture_atlas_handle,
                transform: streamer_transform,
                ..default()
            },
            movement_type: MovementType::Walk,
        },
        streamer_location,
    ));
}

pub fn move_streamer(
    mut streamer_entity: Query<
        (&mut Path, &StartingPoint, &Target, &mut DestinationQueue),
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

    let (mut streamer_path, streamer_tile_pos, streamer_target, mut streamer_destination_queue) =
        streamer_entity
            .get_single_mut()
            .expect("The streamer should be loaded.");

    if !streamer_path.is_empty() || streamer_target.is_some() {
        return;
    }

    if streamer_destination_queue.is_empty() {
        return;
    }

    *streamer_path = get_path(
        &streamer_tile_pos.1,
        &streamer_destination_queue.pop_front().expect(
            "move_streamer: Destination queue for streamer should have been filled with something.",
        ),
        map_size,
        edges,
    );
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
    keyboard_input: Res<Input<KeyCode>>,
    mut destination_request_writer: EventWriter<TilePosEvent>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        destination_request_writer.send(TilePosEvent::new(TilePos { x: 64, y: 52 }));
    }
}

pub fn test_streamer_msg(keyboard_input: Res<Input<KeyCode>>, mut msg_writer: EventWriter<Msg>) {
    if !keyboard_input.just_pressed(KeyCode::Q) {
        return;
    }

    let streamer_msg = Msg {
        speaker_name: "Caveman".to_string(),
        msg: "This is a test message to see if this works and types as expected.".to_string(),
        speaker_role: MovementType::Walk,
    };

    msg_writer.send(streamer_msg);
}
