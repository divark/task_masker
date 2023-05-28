use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

#[derive(Component)]
pub struct NodeData(Vec<Vec2>);

#[derive(Component)]
pub struct NodeEdges(Vec<Vec<usize>>);

#[derive(Bundle)]
pub struct Graph {
    node_data: NodeData,
    node_edges: NodeEdges,
}

fn tilepos_to_idx(x: u32, y: u32, world_size: u32) -> usize {
    ((world_size * x) + y) as usize
}

/// Spawns an Undirected Graph represented by the lowest layer of the map's grid.
// TODO: Figure out how to look at tiles per layer.
pub fn create_fly_graph(tile_positions: Query<&TilePos>, map_information: Query<(&TilemapSize, &TilemapGridSize, &TilemapType)>, mut spawner: Commands) {
    if map_information.is_empty() {
        return;
    }
    
    let (world_size, grid_size, map_type) = map_information.get_single().expect("World size not found. Is it not loaded?");
    
    let mut directed_graph_data: Vec<Vec2> = Vec::new();
    let mut directed_graph_edges: Vec<Vec<usize>> = Vec::new();

    let mut tile_positions_sorted = tile_positions.iter().collect::<Vec<&TilePos>>();
    tile_positions_sorted.sort_by(|&pos1, &pos2| {
        let pos1_converted = tilepos_to_idx(pos1.x, pos1.y, world_size.x);
        let pos2_converted = tilepos_to_idx(pos2.x, pos2.y, world_size.x);

        pos1_converted.cmp(&pos2_converted)
    });

    for tile_position in tile_positions_sorted {
        let tile_idx = tilepos_to_idx(tile_position.x, tile_position.y, world_size.x);
        
        let node_data = tile_position.center_in_world(grid_size, map_type);
        let mut node_edges = Vec::new();

        if tile_position.x > 0 {
            let top_node_idx = tilepos_to_idx(tile_position.x - 1, tile_position.y, world_size.x);
            node_edges.push(top_node_idx);
            directed_graph_edges[top_node_idx].push(tile_idx);
        }

        if tile_position.y > 0 {
            let left_node_idx = tilepos_to_idx(tile_position.x, tile_position.y - 1, world_size.x);
            node_edges.push(left_node_idx);
            directed_graph_edges[left_node_idx].push(tile_idx);
        }

        directed_graph_data.push(node_data);
        directed_graph_edges.push(node_edges);
    }

    spawner.spawn(Graph {
        node_data: NodeData(directed_graph_data),
        node_edges: NodeEdges(directed_graph_edges)
    });
}