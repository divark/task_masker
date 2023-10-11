use std::collections::VecDeque;

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::entities::{streamer::StreamerLabel, MovementType};

use super::{
    plugins::TilePosEvent,
    tiled::{tiledpos_to_tilepos, LayerNumber},
};

#[derive(Component, PartialEq, Debug)]
pub struct Ground;

#[derive(Component)]
pub struct NodeData(Vec<Vec3>);

#[derive(Component, PartialEq, Debug)]
pub struct NodeEdges(Vec<Vec<usize>>);

#[derive(Bundle)]
pub struct Graph {
    node_data: NodeData,
    node_edges: NodeEdges,
}

/// Maps a 2-dimensional (x, y) index into a 1-dimensional array index.
pub fn tilepos_to_idx(x: u32, y: u32, world_size: u32) -> usize {
    ((world_size * x) + y) as usize
}

/// Transforms a 1D array index into its associated 2-dimensional Tilepos index.
pub fn idx_to_tilepos(mapped_idx: usize, world_size: u32) -> TilePos {
    let x = mapped_idx / world_size as usize;
    let y = mapped_idx - (world_size as usize * x);

    TilePos {
        x: x as u32,
        y: y as u32,
    }
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

    let mut height_map: Vec<usize> = vec![0; (world_size.x * world_size.y) as usize];

    // Sorting by Tile Position and Layer number ensures that we won't add
    // a previous node, whether above or to the left, that does not exist
    // yet.
    let mut tile_positions_sorted = tile_positions
        .iter()
        .map(|tile_pair| {
            (
                tiledpos_to_tilepos(tile_pair.0.x, tile_pair.0.y, world_size),
                tile_pair.1,
            )
        })
        .collect::<Vec<(TilePos, &LayerNumber)>>();
    tile_positions_sorted.sort_by(|&pos1, &pos2| {
        let pos1_converted = tilepos_to_idx(pos1.0.x, pos1.0.y, world_size.x);
        let pos2_converted = tilepos_to_idx(pos2.0.x, pos2.0.y, world_size.x);

        let pos1_layer = pos1.1;
        let pos2_layer = pos2.1;

        pos1_converted
            .cmp(&pos2_converted)
            .then(pos1_layer.cmp(pos2_layer))
    });

    for (tile_position, &layer_number) in tile_positions_sorted.iter() {
        let height_idx = tilepos_to_idx(tile_position.x, tile_position.y, world_size.x);
        let height_entry = height_map[height_idx];

        // The difference of 1 here allows for tiles that are not
        // connected to the ground to be ignored. Think scenery
        // that obscures the vision from the camera that the
        // player can pass through.
        if layer_number.0 - height_entry == 1 {
            height_map[height_idx] += 1;
        }
    }

    let mut directed_graph_data: Vec<Vec3> = Vec::new();
    let mut directed_graph_edges: Vec<Vec<usize>> = Vec::new();

    let mut tile_positions_no_layers = tile_positions_sorted
        .iter()
        .map(|x| x.0)
        .collect::<Vec<TilePos>>();
    tile_positions_no_layers.dedup();

    for tile_position in tile_positions_no_layers {
        let tile_idx = tilepos_to_idx(tile_position.x, tile_position.y, world_size.x);
        let tile_height = height_map[tile_idx];

        let tiled_to_bevy_pos = tiledpos_to_tilepos(tile_position.x, tile_position.y, world_size);
        let map_transform = map_information
            .iter()
            .nth(tile_height)
            .expect("Tile should be on this layer.")
            .3;
        let tiled_transform = Transform::from_translation(
            tiled_to_bevy_pos
                .center_in_world(grid_size, map_type)
                .extend(tile_height as f32),
        );
        let node_data = (*map_transform * tiled_transform).translation;
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

        directed_graph_data.push(node_data);
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

#[derive(Component, Deref, DerefMut)]
pub struct Target(pub Option<(Vec3, TilePos)>);

#[derive(Component)]
pub struct StartingPoint(pub Vec3, pub TilePos);

#[derive(Component, Deref, DerefMut)]
pub struct Path(VecDeque<usize>);

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
            node_visited[current_node_idx] = true;
            break;
        }

        if node_visited[current_node_idx] {
            continue;
        }

        node_visited[current_node_idx] = true;

        for node_edge in &graph_node_edges.0[current_node_idx] {
            if node_visited[*node_edge] {
                continue;
            }

            node_parents[*node_edge] = current_node_idx as i32;
            node_distances[*node_edge] = node_distances[current_node_idx] + 1;

            bfs_queue.push_back(*node_edge);
        }
    }

    if !node_visited[mapped_destination_idx] {
        return Path(VecDeque::new());
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

#[derive(Component, PartialEq, PartialOrd, Debug)]
pub enum Direction {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

#[derive(Component, Deref, DerefMut)]
pub struct MovementTimer(pub Timer);

pub fn insert_pathing_information(
    moving_entities: Query<(Entity, &Transform, &TilePos), (With<MovementType>, Without<Path>)>,
    mut spawner: Commands,
) {
    for (moving_entity, entity_transform, entity_tilepos) in &moving_entities {
        spawner.entity(moving_entity).insert((
            Path(VecDeque::new()),
            StartingPoint(entity_transform.translation, *entity_tilepos),
            Target(None),
            Direction::TopRight,
            MovementTimer(Timer::from_seconds(0.05, TimerMode::Repeating)),
        ));
    }
}

pub fn update_movement_target(
    mut moving_entity: Query<(&mut Target, &mut Path, &Transform, &mut Direction)>,
    map_information: Query<&TilemapSize>,
    ground_graph_query: Query<&NodeData, With<Ground>>,
) {
    if ground_graph_query.is_empty() {
        return;
    }

    if map_information.is_empty() {
        return;
    }

    // Each Tile Layer has its own World and Grid size should someone decide
    // to change tilesets for the layer. However, I will not do that, so
    // both the world size and grid size should be the same.
    let world_size = map_information
        .iter()
        .max_by(|&x, &y| {
            let x_world_area = x.x * x.y;
            let y_world_area = y.x * y.y;

            x_world_area.cmp(&y_world_area)
        })
        .expect("Could not find largest world size. Is the map loaded?");

    let nodes = ground_graph_query.get_single().unwrap();
    for (mut target, mut path, current_pos, mut direction) in moving_entity.iter_mut() {
        if target.0.is_none() && !path.0.is_empty() {
            let new_target = path
                .0
                .pop_front()
                .expect("The path was not supposed to be empty by here.");
            let target_tile_pos = idx_to_tilepos(new_target, world_size.y);
            let target_pos = nodes.0[new_target];

            if let Some(new_direction) =
                get_direction(*current_pos, Transform::from_translation(target_pos))
            {
                *direction = new_direction;
            }

            target.0 = Some((target_pos, target_tile_pos));
        }
    }
}

fn get_direction(current_pos: Transform, target_pos: Transform) -> Option<Direction> {
    let current_translation = &current_pos.translation;
    let target_translation = &target_pos.translation;

    if current_translation == target_translation {
        return None;
    }

    let x_direction = target_translation.x - current_translation.x;
    let y_direction = target_translation.y - current_translation.y;

    match (
        x_direction.is_sign_positive(),
        y_direction.is_sign_positive(),
    ) {
        (false, false) => Some(Direction::BottomLeft),
        (true, false) => Some(Direction::BottomRight),
        (false, true) => Some(Direction::TopLeft),
        (true, true) => Some(Direction::TopRight),
    }
}

const NUM_STEPS: f32 = 8.0;

pub fn move_entities(
    mut moving_entity: Query<(
        &mut Transform,
        &mut Target,
        &mut MovementTimer,
        &mut StartingPoint,
    )>,
    time: Res<Time>,
) {
    for (mut current_pos, mut target, mut movement_timer, mut starting_point) in &mut moving_entity
    {
        movement_timer.tick(time.delta());
        if !movement_timer.just_finished() {
            continue;
        }

        if target.0.is_none() {
            continue;
        }

        let (target_vec, target_tile_pos) = target.0.expect("Target should be populated by now.");
        let target_pos = Transform::from_translation(target_vec);

        if *current_pos == target_pos {
            target.0 = None;
            *starting_point = StartingPoint(target_vec, target_tile_pos);
            continue;
        }

        let x_dist = target_pos.translation.x - starting_point.0.x;
        let x_step = x_dist / NUM_STEPS;

        let y_dist = target_pos.translation.y - starting_point.0.y;
        let y_step = y_dist / NUM_STEPS;

        let z_dist = target_pos.translation.z - starting_point.0.z;
        let z_step = z_dist / NUM_STEPS;

        current_pos.translation.x += x_step;
        current_pos.translation.y += y_step;
        current_pos.translation.z += z_step;
    }
}

pub fn update_current_tilepos(
    mut moving_entity: Query<(&mut TilePos, &StartingPoint), Changed<StartingPoint>>,
) {
    for (mut entity_tilepos, entity_starting_point) in &mut moving_entity {
        *entity_tilepos = entity_starting_point.1;
    }
}

pub fn move_streamer(
    mut destination_request_listener: EventReader<TilePosEvent>,
    mut streamer_entity: Query<(&mut Path, &TilePos), With<StreamerLabel>>,
    ground_graph: Query<&NodeEdges, With<Ground>>,
    map_information: Query<(&TilemapSize, &Transform)>,
) {
    if streamer_entity.is_empty() {
        return;
    }

    if ground_graph.is_empty() {
        return;
    }

    let edges = ground_graph
        .get_single()
        .expect("Ground graph should be loaded.");
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

    for destination_request in destination_request_listener.iter() {
        let (mut streamer_path, streamer_tile_pos) = streamer_entity
            .get_single_mut()
            .expect("The streamer should be loaded.");

        *streamer_path = get_path(streamer_tile_pos, destination_request, map_size, edges);
    }
}

pub fn move_streamer_on_spacebar(
    keyboard_input: Res<Input<KeyCode>>,
    mut destination_request_writer: EventWriter<TilePosEvent>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        destination_request_writer.send(TilePosEvent(TilePos { x: 64, y: 52 }));
        //{ x: 64, y: 52 });
    }
}

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
        layers: LayerNumber,
    ) {
        for _layer_num in 0..=layers.0 {
            app.world.spawn_empty().insert((
                map_size,
                grid_size,
                map_type,
                Transform::from_xyz(0.0, 0.0, 0.0),
            ));
        }
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
        spawn_map_information(&mut app, map_size, grid_size, map_type, layer);
        app.add_systems(Update, create_ground_graph);
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
        spawn_map_information(&mut app, map_size, grid_size, map_type, layer);
        app.add_systems(Update, create_ground_graph);
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
        spawn_map_information(&mut app, map_size, grid_size, map_type, layer);
        app.add_systems(Update, create_ground_graph);
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
        spawn_map_information(&mut app, map_size, grid_size, map_type, LayerNumber(2));
        app.add_systems(Update, create_ground_graph);
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
        spawn_map_information(&mut app, map_size, grid_size, map_type, LayerNumber(2));
        app.add_systems(Update, create_ground_graph);
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
        spawn_map_information(&mut app, map_size, grid_size, map_type, LayerNumber(2));
        app.add_systems(Update, create_ground_graph);
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

    #[test]
    fn top_right_direction_and_coordinate_system() {
        let grid_size = TilemapGridSize { x: 32.0, y: 16.0 };

        let map_type = TilemapType::Isometric(IsoCoordSystem::Diamond);

        let source_tilepos = TilePos { x: 0, y: 0 };
        let source_transform = Transform::from_translation(
            source_tilepos
                .center_in_world(&grid_size, &map_type)
                .extend(1.0),
        );
        let destination_tilepos = TilePos { x: 1, y: 1 };
        let destination_transform = Transform::from_translation(
            destination_tilepos
                .center_in_world(&grid_size, &map_type)
                .extend(1.0),
        );

        let expected_direction = Direction::TopRight;
        let actual_direction = get_direction(source_transform, destination_transform).unwrap();

        assert_eq!(expected_direction, actual_direction);
    }

    #[test]
    fn bottom_right_direction_and_coordinate_system() {
        let grid_size = TilemapGridSize { x: 32.0, y: 16.0 };

        let map_type = TilemapType::Isometric(IsoCoordSystem::Diamond);

        let source_tilepos = TilePos { x: 0, y: 0 };
        let source_transform = Transform::from_translation(
            source_tilepos
                .center_in_world(&grid_size, &map_type)
                .extend(1.0),
        );
        let destination_tilepos = TilePos { x: 1, y: 0 };
        let destination_transform = Transform::from_translation(
            destination_tilepos
                .center_in_world(&grid_size, &map_type)
                .extend(1.0),
        );

        let expected_direction = Direction::BottomRight;
        let actual_direction = get_direction(source_transform, destination_transform).unwrap();

        assert_eq!(expected_direction, actual_direction);
    }

    #[test]
    fn top_left_direction_and_coordinate_system() {
        let grid_size = TilemapGridSize { x: 32.0, y: 16.0 };

        let map_type = TilemapType::Isometric(IsoCoordSystem::Diamond);

        let source_tilepos = TilePos { x: 1, y: 0 };
        let source_transform = Transform::from_translation(
            source_tilepos
                .center_in_world(&grid_size, &map_type)
                .extend(1.0),
        );
        let destination_tilepos = TilePos { x: 0, y: 0 };
        let destination_transform = Transform::from_translation(
            destination_tilepos
                .center_in_world(&grid_size, &map_type)
                .extend(1.0),
        );

        let expected_direction = Direction::TopLeft;
        let actual_direction = get_direction(source_transform, destination_transform).unwrap();

        assert_eq!(expected_direction, actual_direction);
    }

    #[test]
    fn bottom_left_direction_and_coordinate_system() {
        let grid_size = TilemapGridSize { x: 32.0, y: 16.0 };

        let map_type = TilemapType::Isometric(IsoCoordSystem::Diamond);

        let source_tilepos = TilePos { x: 0, y: 1 };
        let source_transform = Transform::from_translation(
            source_tilepos
                .center_in_world(&grid_size, &map_type)
                .extend(1.0),
        );
        let destination_tilepos = TilePos { x: 0, y: 0 };
        let destination_transform = Transform::from_translation(
            destination_tilepos
                .center_in_world(&grid_size, &map_type)
                .extend(1.0),
        );

        let expected_direction = Direction::BottomLeft;
        let actual_direction = get_direction(source_transform, destination_transform).unwrap();

        assert_eq!(expected_direction, actual_direction);
    }
}
