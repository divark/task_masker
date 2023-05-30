use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use super::tiled::LayerNumber;

#[derive(Component)]
pub struct Ground;

#[derive(Component)]
pub struct NodeData(Vec<Vec2>);

#[derive(Component)]
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
