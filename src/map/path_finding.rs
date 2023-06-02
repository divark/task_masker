use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use super::tiled::LayerNumber;

#[derive(Component, PartialEq, Debug)]
pub struct Ground;

#[derive(Component)]
pub struct NodeData(Vec<Vec2>);

#[derive(Component, PartialEq, Debug)]
pub struct NodeEdges(Vec<Vec<usize>>);

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
    map_information: Query<(&TilemapSize, &TilemapGridSize, &TilemapType)>,
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

    let mut directed_graph_data: Vec<Vec2> = Vec::new();
    let mut directed_graph_edges: Vec<Vec<usize>> = Vec::new();

    let mut tile_positions_no_layers = tile_positions_sorted
        .iter()
        .map(|x| x.0)
        .collect::<Vec<&TilePos>>();
    tile_positions_no_layers.dedup();

    for tile_position in tile_positions_no_layers {
        let tile_idx = tilepos_to_idx(tile_position.x, tile_position.y, world_size.x);
        let tile_height = height_map[tile_idx];

        let node_data = tile_position.center_in_world(grid_size, map_type);
        let mut node_edges = Vec::new();

        if tile_position.x > 0 && tile_height > 0 {
            let top_node_idx = tilepos_to_idx(tile_position.x - 1, tile_position.y, world_size.x);
            let height_difference: i32 = height_map[top_node_idx] - tile_height;
            if height_difference.abs() <= 1 {
                node_edges.push(top_node_idx);
                directed_graph_edges[top_node_idx].push(tile_idx);
            }
        }

        if tile_position.y > 0 && tile_height > 0 {
            let left_node_idx = tilepos_to_idx(tile_position.x, tile_position.y - 1, world_size.x);
            let height_difference: i32 = height_map[left_node_idx] - tile_height;
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

#[derive(Component)]
pub struct PathDistance(Vec<i32>);

#[derive(Component)]
pub struct PathParent(Vec<i32>);

/// Attaches the distances and parent nodes for all shortest paths to a recently added
/// Undirected Graph using the Floyd-Warshall All-Pairs Shortest Path algorithm.
/// https://en.wikipedia.org/wiki/Floyd%E2%80%93Warshall_algorithm
pub fn create_shortest_paths_all_pairs(added_graph: Query<(Entity, &NodeEdges), Added<NodeEdges>>, mut spawner: Commands) {
    for (entity, graph_edges) in added_graph.iter() {
        let node_edges = &graph_edges.0;
        let num_nodes = graph_edges.0.len();

        let mut parent_of = vec![-1; num_nodes * num_nodes];
        let mut distance = vec![i32::MAX; num_nodes * num_nodes];

        for node_idx in 0..num_nodes {
            let mapped_1d_idx = tilepos_to_idx(node_idx as u32, node_idx as u32, num_nodes as u32);

            distance[mapped_1d_idx] = 0;
            parent_of[mapped_1d_idx] = node_idx as i32;
        }

        for node_idx in 0..num_nodes {
            for edge_idx in &node_edges[node_idx] {
                let mapped_1d_idx = tilepos_to_idx(node_idx as u32, *edge_idx as u32, num_nodes as u32);

                distance[mapped_1d_idx] = 1;
                parent_of[mapped_1d_idx] = node_idx as i32;
            }
        }

        for num_nodes_to_consider in 0..num_nodes {
            for first_node in 0..num_nodes {
                for second_node in 0..num_nodes {
                    let mapped_first_to_second_idx = tilepos_to_idx(first_node as u32, second_node as u32, num_nodes as u32);
                    let mapped_first_to_middle_idx = tilepos_to_idx(first_node as u32, num_nodes_to_consider as u32, num_nodes as u32);
                    let mapped_middle_to_second_idx = tilepos_to_idx(num_nodes_to_consider as u32, second_node as u32, num_nodes as u32);

                    let direct_distance = distance[mapped_first_to_second_idx];
                    let first_to_middle_distance = distance[mapped_first_to_middle_idx];
                    let middle_to_second_distance = distance[mapped_middle_to_second_idx];

                    if first_to_middle_distance == i32::MAX || middle_to_second_distance == i32::MAX {
                        continue;
                    }

                    if direct_distance > first_to_middle_distance + middle_to_second_distance {
                        distance[mapped_first_to_second_idx] = distance[mapped_first_to_middle_idx] + distance[mapped_middle_to_second_idx];
                        parent_of[mapped_first_to_second_idx] = parent_of[mapped_middle_to_second_idx];
                    }
                }
            }
        }

        spawner.entity(entity)
            .insert(
                (
                    PathDistance(distance),
                    PathParent(parent_of)
                )
            );
    }
}

#[derive(Component)]
pub struct Target(Option<Vec2>);

#[derive(Component)]
pub struct Path(Vec<Vec2>);

pub fn get_path(source: &TilePos, destination: &TilePos, map_size: &TilemapSize, graph_path_distances: &PathDistance, graph_path_parents: &PathParent, graph_node_data: &NodeData) -> Path {
    let num_edges = graph_path_distances.0.len();
    let num_nodes = f64::sqrt(num_edges as f64) as usize;

    let mapped_source_idx = tilepos_to_idx(source.x, source.y, map_size.y);
    let mut mapped_destination_idx = tilepos_to_idx(destination.x, destination.y, map_size.y);

    let mut mapped_path_idx = tilepos_to_idx(mapped_source_idx as u32, mapped_destination_idx as u32, num_nodes as u32);
    
    let parent_of = &graph_path_parents.0;
    if parent_of[mapped_path_idx] == -1 {
        return Path(Vec::new());
    }

    let nodes = &graph_node_data.0;
    let mut path = vec![nodes[mapped_destination_idx]];
    while mapped_source_idx != mapped_destination_idx {
        mapped_path_idx = tilepos_to_idx(mapped_source_idx as u32, mapped_destination_idx as u32, num_nodes as u32);
        mapped_destination_idx = parent_of[mapped_path_idx] as usize;

        path.push(nodes[mapped_destination_idx]);
    }

    path.reverse();

    Path(path)
}

pub fn update_movement_target(mut moving_entity: Query<(&mut Target, &mut Path)>) {
    for (mut target, mut path) in moving_entity.iter_mut() {
        if target.0.is_none() && !path.0.is_empty() {
            let new_target = path.0.pop().expect("The path was not supposed to be empty by here.");
            target.0 = Some(new_target);
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

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
    fn triangle_cycle_graph_pathfinding() {
        let node_edges = NodeEdges(
            vec![
                vec![1, 2],
                vec![0, 2],
                vec![0, 1]
            ]
        );

        let num_nodes = node_edges.0.len();

        let mut app = App::new();
        app.add_system(create_shortest_paths_all_pairs);
        app.world.spawn_empty().insert(node_edges);
        app.update();

        let (path_distances, path_parents) = app
            .world
            .query::<(&PathDistance, &PathParent)>().get_single(&app.world)
            .expect("Could not find path distances and parents");

        for i in 0..num_nodes {
            for j in 0..num_nodes {
                let mapped_idx = tilepos_to_idx(i as u32, j as u32, num_nodes as u32);
                let expected_distance = if i == j {
                    0
                } else {
                    1
                };

                assert_eq!(expected_distance, path_distances.0[mapped_idx], "Nodes {} and {}", i, j);
                assert_eq!(i, path_parents.0[mapped_idx] as usize, "Nodes {} and {}", i, j);
            }
        }
    }

    #[test]
    fn triangle_graph_pathfinding() {
        let node_edges = NodeEdges(
            vec![
                vec![1],
                vec![0, 2],
                vec![1]
            ]
        );

        let num_nodes = node_edges.0.len();

        let mut app = App::new();
        app.add_system(create_shortest_paths_all_pairs);
        app.world.spawn_empty().insert(node_edges);
        app.update();

        let (path_distances, path_parents) = app
            .world
            .query::<(&PathDistance, &PathParent)>().get_single(&app.world)
            .expect("Could not find path distances and parents");

        let mut expected_parents = vec![vec![-1; num_nodes]; num_nodes];
        // Direct Edges
        expected_parents[0][0] = 0;
        expected_parents[0][1] = 0;
        expected_parents[1][0] = 1;

        expected_parents[1][1] = 1;
        expected_parents[1][2] = 1;

        expected_parents[2][1] = 2;
        expected_parents[2][2] = 2;

        // Paths
        expected_parents[0][2] = 1;
        expected_parents[2][0] = 1;

        for i in 0..num_nodes {
            for j in 0..num_nodes {
                let mapped_idx = tilepos_to_idx(i as u32, j as u32, num_nodes as u32);
                let expected_distance = if i == j {
                    0
                } else if (i == 0 && j == 2) || (i == 2 && j == 0) {
                    2
                } else {
                    1
                };

                assert_eq!(expected_distance, path_distances.0[mapped_idx], "Nodes {} and {}", i, j);
                assert_eq!(expected_parents[i][j], path_parents.0[mapped_idx], "Nodes {} and {}", i, j);
            }
        }
    }
}
