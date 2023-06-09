//! In Path-Finding, there are two components that are important:
//! 1. Mapping the Map into an Undirected Graph
//! 2. Returning the Shortest Path for some Start and End Node.
//!
//! With these two ideas, the first thing to do is to consider
//! how we're going to represent an Undirected Graph, and then
//! how we'll translate whatever 2D map we have to it.

use std::collections::VecDeque;

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::entities::{streamer::StreamerLabel, MovementType};

use super::tiled::{tiledpos_to_tilepos, LayerNumber};

/// Labels a Graph to be processed for entities
/// that walk, such as the Streamer in this case.
#[derive(Component, PartialEq, Debug)]
pub struct Ground;

/// Holds a Destination Tile Position morphed from
/// Tiled coordinates to Bevy Coordinates for each
/// node.
#[derive(Component)]
pub struct NodeData(Vec<TilePos>);

/// Contains a list of Nodes adjacent to some
/// source node in terms of indices. Identical
/// to the Adjacency List pattern for graphs.
#[derive(Component, PartialEq, Debug)]
pub struct NodeEdges(Vec<Vec<usize>>);

/// A Directed Graph implementation constructed
/// in a Data-Oriented Fashion, consisting of just two
/// 1 dimensional arrays, where a Node is just some
/// index (Entity).
#[derive(Bundle)]
pub struct Graph {
    node_data: NodeData,
    node_edges: NodeEdges,
}

/// Maps a 2-dimensional (x, y) index into a 1-dimensional array index.
fn tilepos_to_idx(x: u32, y: u32, world_size: u32) -> usize {
    ((world_size * x) + y) as usize
}

/// Spawns an Undirected Graph representing all land titles where the edges
/// indicate an at most 1 tile offset between two tiles.
pub fn create_ground_graph(
    tile_positions: Query<(&TilePos, &LayerNumber)>,
    map_information: Query<(&TilemapSize, &TilemapGridSize, &TilemapType, &Transform)>,
    ground_graph_query: Query<(&NodeEdges, &NodeData), With<Ground>>,
    mut spawner: Commands,
) {
    if map_information.is_empty() {
        return;
    }

    if !ground_graph_query.is_empty() {
        return;
    }

    // Each Tile Layer has its own World and Grid size should someone decide
    // to change tilesets for the layer. However, I will not do that, so
    // both the world size and grid size should be the same.
    let world_size = map_information
        .iter()
        .map(|sizes| sizes.0)
        .max_by(|&x, &y| {
            let x_world_area = x.x * x.y;
            let y_world_area = y.x * y.y;

            x_world_area.cmp(&y_world_area)
        })
        .expect("Could not find largest world size. Is the map loaded?");

    let grid_size = map_information
        .iter()
        .map(|sizes| sizes.1)
        .max_by(|&x, &y| {
            let x_grid_area = x.x * x.y;
            let y_grid_area = y.x * y.y;

            x_grid_area.partial_cmp(&y_grid_area).unwrap()
        })
        .expect("Could not find largest grid size. Is the map loaded?");

    // I'm not sure how a map type could change based on the layer, but
    // the Tiled loader insists it could happen, but I won't do that for
    // my own maps, so they should all be equal.
    let map_type = map_information
        .iter()
        .next()
        .map(|x| x.2)
        .expect("Could not get map type. Is the map loaded?");

    let mut height_map = vec![0; (world_size.x * world_size.y) as usize];

    // Sorting by Tile Position and Layer number ensures that we won't add
    // a previous node, whether above or to the left, that does not exist
    // yet.
    let mut tile_positions_sorted = tile_positions
        .iter()
        .collect::<Vec<(&TilePos, &LayerNumber)>>();
    tile_positions_sorted.sort_by(|&pos1, &pos2| {
        let pos1_converted = tilepos_to_idx(pos1.0.x, pos1.0.y, world_size.x);
        let pos2_converted = tilepos_to_idx(pos2.0.x, pos2.0.y, world_size.x);

        let pos1_layer = pos1.1;
        let pos2_layer = pos2.1;

        pos1_converted
            .cmp(&pos2_converted)
            .then(pos1_layer.cmp(pos2_layer))
    });

    for (&tile_position, &layer_number) in tile_positions_sorted.iter() {
        let height_idx = tilepos_to_idx(tile_position.x, tile_position.y, world_size.x);
        let height_entry = height_map[height_idx];

        // The difference of 1 here allows for tiles that are not
        // connected to the ground to be ignored. Think scenery
        // that obscures the vision from the camera that the
        // player can pass through.
        if layer_number.0 - (height_entry as usize) == 1 {
            height_map[height_idx] += 1;
        }
    }

    let mut directed_graph_data: Vec<TilePos> = Vec::new();
    let mut directed_graph_edges: Vec<Vec<usize>> = Vec::new();

    let mut tile_positions_no_layers = tile_positions_sorted
        .iter()
        .map(|x| x.0)
        .collect::<Vec<&TilePos>>();
    tile_positions_no_layers.dedup();

    for tile_position in tile_positions_no_layers {
        let tile_idx = tilepos_to_idx(tile_position.x, tile_position.y, world_size.x);
        let tile_height = height_map[tile_idx];

        let tiled_pos = tiledpos_to_tilepos(tile_position.x, tile_position.y, &world_size);
        let mut node_edges = Vec::new();

        if tile_position.x > 0 && tile_height > 0 {
            let top_node_idx = tilepos_to_idx(tile_position.x - 1, tile_position.y, world_size.x);
            let height_difference: i32 = height_map[top_node_idx] as i32 - tile_height as i32;
            if height_difference.abs() <= 1 {
                node_edges.push(top_node_idx);
                directed_graph_edges[top_node_idx].push(tile_idx);
            }
        }

        if tile_position.y > 0 && tile_height > 0 {
            let left_node_idx = tilepos_to_idx(tile_position.x, tile_position.y - 1, world_size.x);
            let height_difference: i32 = height_map[left_node_idx] as i32 - tile_height as i32;
            if height_difference.abs() <= 1 {
                node_edges.push(left_node_idx);
                directed_graph_edges[left_node_idx].push(tile_idx);
            }
        }

        directed_graph_data.push(tiled_pos);
        directed_graph_edges.push(node_edges);
    }

    spawner.spawn((
        Graph {
            node_data: NodeData(directed_graph_data),
            node_edges: NodeEdges(directed_graph_edges),
        },
        Ground,
    ));
}

/// Holds a Queue of Node indices (Entities) as the result
/// of some Shortest Path computation.
#[derive(Component)]
pub struct Path(VecDeque<usize>);

/// Returns a Path found for some Source and Destination Tiled-based
/// Tile Positions.
pub fn get_path(
    source: &TilePos,
    destination: &TilePos,
    map_size: &TilemapSize,
    graph_node_edges: &NodeEdges,
) -> Path {
    let mut node_distances = vec![0; graph_node_edges.0.len()];
    let mut node_parents = vec![-1; graph_node_edges.0.len()];
    let mut node_visited = vec![false; graph_node_edges.0.len()];

    let mapped_source_idx = tilepos_to_idx(source.x, source.y, map_size.y);
    let mut mapped_destination_idx = tilepos_to_idx(destination.x, destination.y, map_size.y);

    let mut bfs_queue = VecDeque::from([mapped_source_idx]);
    while !bfs_queue.is_empty() {
        let current_node_idx = bfs_queue.pop_front().unwrap();
        if current_node_idx == mapped_destination_idx {
            break;
        }

        if node_visited[current_node_idx] {
            continue;
        }

        node_visited[current_node_idx] = true;

        for node_edge in &graph_node_edges.0[current_node_idx] {
            node_parents[*node_edge] = current_node_idx as i32;
            node_distances[*node_edge] = node_distances[current_node_idx] + 1;

            bfs_queue.push_front(*node_edge);
        }
    }

    let mut path = VecDeque::from([mapped_destination_idx]);
    while mapped_source_idx != mapped_destination_idx {
        let node_parent = node_parents[mapped_destination_idx];
        if node_parent == -1 {
            break;
        }

        mapped_destination_idx = node_parent as usize;

        path.push_front(mapped_destination_idx);
    }

    Path(path)
}

/*
* Okay, so now comes the fun part: Actually moving entities
* from some Path we've computed.
 */
// TODO: Should this be a message?
#[derive(Component)]
pub struct TileTarget(Option<TilePos>);

/// Holds a World Position (Transform) as a destination
/// for some on-going move request for some Entity.
#[derive(Component)]
pub struct TransformTarget(Option<Vec2>);

/// Holds the number of pixels that should be moved on
/// the x and y axis for some timer interval.
#[derive(Component)]
pub struct PixelsPerTime {
    timer: Timer,
    x_rate: f32,
    y_rate: f32,
}

/// Sets up any moving-based entities to have the information necessary
/// for Path-Finding to work.
pub fn attach_pathing_info(
    moving_entities: Query<Entity, (With<MovementType>, Without<Path>)>,
    mut spawner: Commands,
) {
    for moving_entity in &moving_entities {
        spawner.entity(moving_entity).insert((
            Path(VecDeque::new()),
            TileTarget(None),
            TransformTarget(None),
        ));
    }
}

/// Populates Shortest Path based off of Streamer's Tile Position (or current Target)
/// to some Destination Tile Position.
pub fn move_streamer(
    mut destination_requests: EventReader<TilePos>,
    mut streamer_entity: Query<(&TilePos, &TileTarget, &mut Path), With<MovementType>>,
    map_size_query: Query<&TilemapSize>,
    ground_graph: Query<&NodeEdges, With<Ground>>,
) {
    if ground_graph.is_empty() {
        return;
    }

    if map_size_query.is_empty() {
        return;
    }

    if streamer_entity.is_empty() {
        return;
    }

    let map_size = map_size_query
        .iter()
        .nth(1)
        .expect("Tiled map information should be loaded by now.");

    let (streamer_tile_pos, streamer_tile_target, mut streamer_path) = streamer_entity
        .get_single_mut()
        .expect("There should be one Streamer spawned.");

    let ground_node_edges = ground_graph
        .get_single()
        .expect("There should be one Ground Graph spawned by this point.");

    for destination_tile_pos in destination_requests.iter() {
        if let Some(past_target_tile_pos) = &streamer_tile_target.0 {
            *streamer_path = get_path(
                past_target_tile_pos,
                destination_tile_pos,
                map_size,
                ground_node_edges,
            );
        } else {
            *streamer_path = get_path(
                streamer_tile_pos,
                destination_tile_pos,
                map_size,
                ground_node_edges,
            );
        }
    }
}

pub fn set_new_target(
    mut moving_entities: Query<
        (&TilePos, &mut TransformTarget, &mut TileTarget, &mut Path),
        Without<PixelsPerTime>,
    >,
    node_data: Query<&NodeData>,
    map_information: Query<
        (&Transform, &TilemapType, &TilemapGridSize, &TilemapSize),
        (Added<TilemapType>, Added<TilemapGridSize>),
    >,
) {
    if node_data.is_empty() {
        return;
    }

    if moving_entities.is_empty() {
        return;
    }

    let (map_transform, map_type, grid_size, map_size) = map_information
        .iter()
        .nth(1)
        .expect("Could not load map information. Is world loaded?");

    for (entity_tile_pos, mut entity_transform, mut entity_target_tile_pos, mut entity_path) in
        &mut moving_entities
    {
        let streamer_translation = entity_tile_pos
            .center_in_world(grid_size, map_type)
            .extend(20.0);
        let streamer_transform = *map_transform * Transform::from_translation(streamer_translation);
    }
}

// #[derive(Component, PartialEq, PartialOrd, Debug)]
// pub enum Direction {
//     TopLeft,
//     TopRight,
//     BottomLeft,
//     BottomRight,
// }

// fn get_direction(current_pos: Transform, target_pos: Transform) -> Direction {
//     let current_translation = &current_pos.translation;
//     let target_translation = &target_pos.translation;

//     let x_direction = target_translation.x > current_translation.x;
//     let y_direction = target_translation.y > current_translation.y;

//     match (x_direction, y_direction) {
//         (true, true) => Direction::BottomLeft,
//         (true, false) => Direction::BottomRight,
//         (false, true) => Direction::TopLeft,
//         (false, false) => Direction::TopRight,
//     }
// }

#[cfg(test)]
pub mod tests {
    use super::{Direction, *};

    const TILE_WIDTH_PX: f32 = 32.0;
    const TILE_LENGTH_PX: f32 = 32.0;

    fn spawn_tiles(app: &mut App, length: u32, width: u32, layer: &LayerNumber) {
        for i in 0..length {
            for j in 0..width {
                app.world.spawn_empty().insert((TilePos::new(i, j), *layer));
            }
        }
    }

    fn spawn_map_information(
        app: &mut App,
        map_size: TilemapSize,
        grid_size: TilemapGridSize,
        map_type: TilemapType,
    ) {
        app.world
            .spawn_empty()
            .insert((map_size, grid_size, map_type));
    }

    #[test]
    fn two_by_two_ground_tiles() {
        let layer = LayerNumber(1);
        let map_size = TilemapSize { x: 2, y: 2 };

        let grid_size = TilemapGridSize {
            x: TILE_LENGTH_PX,
            y: TILE_WIDTH_PX,
        };

        let map_type = TilemapType::Isometric(IsoCoordSystem::Diamond);

        let mut app = App::new();
        spawn_tiles(&mut app, map_size.x, map_size.y, &layer);
        spawn_map_information(&mut app, map_size, grid_size, map_type);
        app.add_system(create_ground_graph);
        app.update();

        let expected_node_edges = NodeEdges(vec![vec![1, 2], vec![0, 3], vec![0, 3], vec![1, 2]]);
        let expected_graph_type = Ground;

        let mut graph_query = app.world.query::<(&NodeEdges, &Ground)>();
        let (actual_node_edges, actual_graph_type) = graph_query.single(&app.world);

        assert_eq!(expected_node_edges, *actual_node_edges);
        assert_eq!(expected_graph_type, *actual_graph_type);
    }

    #[test]
    fn only_one_ground_graph() {
        let layer = LayerNumber(1);
        let map_size = TilemapSize { x: 2, y: 2 };

        let grid_size = TilemapGridSize {
            x: TILE_LENGTH_PX,
            y: TILE_WIDTH_PX,
        };

        let map_type = TilemapType::Isometric(IsoCoordSystem::Diamond);

        let mut app = App::new();
        spawn_tiles(&mut app, map_size.x, map_size.y, &layer);
        spawn_map_information(&mut app, map_size, grid_size, map_type);
        app.add_system(create_ground_graph);
        app.update();
        app.update();

        let mut graph_query = app.world.query::<(&NodeEdges, &Ground)>();
        assert_eq!(graph_query.iter(&app.world).count(), 1);
    }

    #[test]
    fn two_by_two_no_ground_tiles() {
        let layer = LayerNumber(0);
        let map_size = TilemapSize { x: 2, y: 2 };

        let grid_size = TilemapGridSize {
            x: TILE_LENGTH_PX,
            y: TILE_WIDTH_PX,
        };

        let map_type = TilemapType::Isometric(IsoCoordSystem::Diamond);

        let mut app = App::new();
        spawn_tiles(&mut app, map_size.x, map_size.y, &layer);
        spawn_map_information(&mut app, map_size, grid_size, map_type);
        app.add_system(create_ground_graph);
        app.update();

        let expected_node_edges = NodeEdges(vec![vec![], vec![], vec![], vec![]]);
        let expected_graph_type = Ground;

        let mut graph_query = app.world.query::<(&NodeEdges, &Ground)>();
        let (actual_node_edges, actual_graph_type) = graph_query.single(&app.world);

        assert_eq!(expected_node_edges, *actual_node_edges);
        assert_eq!(expected_graph_type, *actual_graph_type);
    }

    #[test]
    fn two_by_two_ground_tiles_all_raised() {
        let layer = LayerNumber(1);
        let map_size = TilemapSize { x: 2, y: 2 };

        let grid_size = TilemapGridSize {
            x: TILE_LENGTH_PX,
            y: TILE_WIDTH_PX,
        };

        let map_type = TilemapType::Isometric(IsoCoordSystem::Diamond);

        let mut app = App::new();
        spawn_tiles(&mut app, map_size.x, map_size.y, &layer);
        spawn_tiles(&mut app, map_size.x, map_size.y, &LayerNumber(2));
        spawn_map_information(&mut app, map_size, grid_size, map_type);
        app.add_system(create_ground_graph);
        app.update();

        let expected_node_edges = NodeEdges(vec![vec![1, 2], vec![0, 3], vec![0, 3], vec![1, 2]]);
        let expected_graph_type = Ground;

        let mut graph_query = app.world.query::<(&NodeEdges, &Ground)>();
        let (actual_node_edges, actual_graph_type) = graph_query.single(&app.world);

        assert_eq!(expected_node_edges, *actual_node_edges);
        assert_eq!(expected_graph_type, *actual_graph_type);
    }

    #[test]
    fn two_by_two_ground_tiles_right_corner_raised() {
        let layer = LayerNumber(1);
        let map_size = TilemapSize { x: 2, y: 2 };

        let grid_size = TilemapGridSize {
            x: TILE_LENGTH_PX,
            y: TILE_WIDTH_PX,
        };

        let map_type = TilemapType::Isometric(IsoCoordSystem::Diamond);

        let mut app = App::new();
        spawn_tiles(&mut app, map_size.x, map_size.y, &layer);
        app.world
            .spawn_empty()
            .insert((TilePos::new(1, 1), LayerNumber(2)));
        spawn_map_information(&mut app, map_size, grid_size, map_type);
        app.add_system(create_ground_graph);
        app.update();

        let expected_node_edges = NodeEdges(vec![vec![1, 2], vec![0, 3], vec![0, 3], vec![1, 2]]);
        let expected_graph_type = Ground;

        let mut graph_query = app.world.query::<(&NodeEdges, &Ground)>();
        let (actual_node_edges, actual_graph_type) = graph_query.single(&app.world);

        assert_eq!(expected_node_edges, *actual_node_edges);
        assert_eq!(expected_graph_type, *actual_graph_type);
    }

    #[test]
    fn two_by_two_ground_tiles_left_corner_raised() {
        let layer = LayerNumber(1);
        let map_size = TilemapSize { x: 2, y: 2 };

        let grid_size = TilemapGridSize {
            x: TILE_LENGTH_PX,
            y: TILE_WIDTH_PX,
        };

        let map_type = TilemapType::Isometric(IsoCoordSystem::Diamond);

        let mut app = App::new();
        spawn_tiles(&mut app, map_size.x, map_size.y, &layer);
        app.world
            .spawn_empty()
            .insert((TilePos::new(0, 0), LayerNumber(2)));
        spawn_map_information(&mut app, map_size, grid_size, map_type);
        app.add_system(create_ground_graph);
        app.update();

        let expected_node_edges = NodeEdges(vec![vec![1, 2], vec![0, 3], vec![0, 3], vec![1, 2]]);
        let expected_graph_type = Ground;

        let mut graph_query = app.world.query::<(&NodeEdges, &Ground)>();
        let (actual_node_edges, actual_graph_type) = graph_query.single(&app.world);

        assert_eq!(expected_node_edges, *actual_node_edges);
        assert_eq!(expected_graph_type, *actual_graph_type);
    }

    #[test]
    fn triangle_graph_pathfinding_paths() {
        let node_edges = NodeEdges(vec![vec![1], vec![0, 2], vec![1]]);

        let node_positions = vec![TilePos::new(0, 0), TilePos::new(0, 1), TilePos::new(0, 2)];

        let world_size = TilemapSize { x: 1, y: 3 };

        let num_nodes = node_edges.0.len();

        let mut expected_paths = vec![vec![vec![]; num_nodes]; num_nodes];
        expected_paths[0][0] = vec![0];
        expected_paths[1][1] = vec![1];
        expected_paths[2][2] = vec![2];

        expected_paths[0][1] = vec![0, 1];
        expected_paths[1][0] = vec![1, 0];
        expected_paths[1][2] = vec![1, 2];
        expected_paths[2][1] = vec![2, 1];

        expected_paths[0][2] = vec![0, 1, 2];
        expected_paths[2][0] = vec![2, 1, 0];

        for start_pos in &node_positions {
            let mapped_start_idx = tilepos_to_idx(start_pos.x, start_pos.y, world_size.y);
            for end_pos in &node_positions {
                let mapped_end_idx = tilepos_to_idx(end_pos.x, end_pos.y, world_size.y);

                assert_eq!(
                    VecDeque::from(expected_paths[mapped_start_idx][mapped_end_idx].clone()),
                    get_path(start_pos, end_pos, &world_size, &node_edges,).0,
                    "Nodes {} and {}",
                    mapped_start_idx,
                    mapped_end_idx
                );
            }
        }
    }

    #[test]
    fn tilepos_to_world_transform_and_back() {
        let map_size = TilemapSize { x: 4, y: 4 };

        let grid_size = TilemapGridSize { x: 32.0, y: 16.0 };

        let map_type = TilemapType::Isometric(IsoCoordSystem::Diamond);

        let tilepos = TilePos { x: 1, y: 1 };
        let tile_transform =
            Transform::from_translation(tilepos.center_in_world(&grid_size, &map_type).extend(1.0));

        let tilepos_from_transform = TilePos::from_world_pos(
            &tile_transform.translation.truncate(),
            &map_size,
            &grid_size,
            &map_type,
        );

        assert!(tilepos_from_transform.is_some());
        let actual_tilepos = tilepos_from_transform.unwrap();

        assert_eq!(tilepos, actual_tilepos);
    }
}
