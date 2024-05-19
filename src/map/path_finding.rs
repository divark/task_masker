use std::collections::{HashSet, VecDeque};

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::entities::{subscriber::SUBSCRIBER_LAYER_NUM, MovementType};

use super::tiled::{
    flip_y_axis_for_tile_pos, tiled_to_bevy_transform, to_bevy_transform, LayerNumber,
    TiledMapInformation,
};

#[derive(Component, PartialEq, Debug)]
pub enum GraphType {
    Ground,
    Air,
    Water,
}

pub struct TranslationGatherer {
    map_information: Vec<(TilemapGridSize, TilemapType, Transform)>,
}

impl TranslationGatherer {
    pub fn new(map_information: Vec<(TilemapGridSize, TilemapType, Transform)>) -> Self {
        Self { map_information }
    }

    /// Returns a Transform (Position) for each Heighted Tile.
    pub fn translations_from(&self, heighted_tiles: &Vec<HeightedTilePos>) -> Vec<Vec3> {
        let mut heighted_tile_translations = Vec::new();

        let tile_height_map = height_map_from(heighted_tiles);
        let unique_tiles = unique_tiles_from(heighted_tiles);
        let (length, _width, _height) = dimensions_from(heighted_tiles);

        for tile in unique_tiles {
            let tile_idx = tilepos_to_idx(tile.x, tile.y, length);
            let tile_height = tile_height_map[tile_idx];

            let (grid_size, map_type, map_transform) =
                self.map_information.iter().nth(tile_height).expect(
                    "translations_from: Could not find map information at given tile height.",
                );

            let tile_translation = tile
                .center_in_world(grid_size, map_type)
                .extend(map_transform.translation.z);
            let tile_transform = *map_transform * Transform::from_translation(tile_translation);

            heighted_tile_translations.push(tile_transform.translation);
        }

        heighted_tile_translations
    }
}

#[derive(Component)]
pub struct NodeData(pub Vec<Vec3>);

impl NodeData {
    pub fn from_ground_tiles(
        heighted_tiles: &Vec<HeightedTilePos>,
        layer_map_information: Vec<(TilemapGridSize, TilemapType, Transform)>,
    ) -> Self {
        let translation_gatherer = TranslationGatherer::new(layer_map_information);

        let tile_translations = translation_gatherer.translations_from(heighted_tiles);

        NodeData(tile_translations)
    }
}

#[derive(Component, PartialEq, Debug)]
pub struct NodeEdges(pub Vec<Vec<usize>>);

/// Returns the Length, Width, and Height derived from
/// some collection of Heighted Tile Positions.
fn dimensions_from(heighted_tiles: &Vec<HeightedTilePos>) -> (u32, u32, u32) {
    let (mut min_x, mut max_x) = (0, 0);
    let (mut min_y, mut max_y) = (0, 0);
    let (mut min_z, mut max_z) = (0, 0);

    for heighted_tile in heighted_tiles {
        let x = heighted_tile.x();
        let y = heighted_tile.y();
        let z = heighted_tile.z();

        min_x = min_x.min(x);
        max_x = max_x.max(x);

        min_y = min_y.min(y);
        max_y = max_x.max(y);

        min_z = min_z.min(z);
        max_z = max_z.max(z);
    }

    let length = max_x - min_x + 1;
    let width = max_y - min_y + 1;
    let height = max_z - min_z + 1;

    (length, width, height)
}

/// Returns a collection of Heighted Tiles with their y-axes
/// flipped.
fn flip_heighted_tiles(heighted_tiles: &Vec<HeightedTilePos>) -> Vec<HeightedTilePos> {
    let (_length, width, _height) = dimensions_from(heighted_tiles);

    heighted_tiles
        .iter()
        .map(|tile| tile.flip(width))
        .collect::<Vec<HeightedTilePos>>()
}

/// Returns a 1 Dimensional Height Map calculated from some
/// collection of Heighted Tile Positions with respect to
/// the desired length and width.
fn height_map_from(ground_tiles: &Vec<HeightedTilePos>) -> Vec<usize> {
    let (length, width, _height) = dimensions_from(ground_tiles);
    let mut height_map: Vec<usize> = vec![0; length as usize * width as usize];

    // Sorting by Tile Position and Layer number ensures that we won't add
    // a previous node, whether above or to the left, that does not exist
    // yet.
    let mut ground_tiles_sorted = ground_tiles.to_vec();
    ground_tiles_sorted.sort_unstable();

    for heighted_tile in ground_tiles_sorted.iter() {
        let height_idx = tilepos_to_idx(heighted_tile.x(), heighted_tile.y(), length);
        let height_entry = height_map[height_idx];

        // The difference of 1 here allows for tiles that are not
        // connected to the ground to be ignored. Think scenery
        // that obscures the vision from the camera that the
        // player can pass through.
        let tile_height = heighted_tile.z() as usize;
        if tile_height - height_entry == 1 {
            height_map[height_idx] += 1;
        }
    }

    height_map
}

/// Returns a collection of Tile Positions sorted and extracted
/// from Heighted Tile Positions.
pub fn unique_tiles_from(tiles: &Vec<HeightedTilePos>) -> Vec<TilePos> {
    let mut unique_tiles = HashSet::new();

    for tile in tiles {
        unique_tiles.insert(tile.xy);
    }

    let mut unique_tiles_no_dups = Vec::from_iter(unique_tiles);
    unique_tiles_no_dups
        .sort_by(|source, other| source.x.cmp(&other.x).then(source.y.cmp(&other.y)));

    unique_tiles_no_dups
}

impl NodeEdges {
    /// Returns a set of Node Edges derived from a collection of Tiles
    /// designated for Ground traversal.
    pub fn from_ground_tiles(ground_tiles: Vec<HeightedTilePos>) -> NodeEdges {
        let mut directed_graph_edges: Vec<Vec<usize>> = Vec::with_capacity(ground_tiles.len());

        let height_map: Vec<usize> = height_map_from(&ground_tiles);
        let (length, _width, _height) = dimensions_from(&ground_tiles);

        let tile_positions_no_layers = unique_tiles_from(&ground_tiles);

        for tile_position in tile_positions_no_layers {
            let tile_idx = tilepos_to_idx(tile_position.x, tile_position.y, length);
            let tile_height = height_map[tile_idx];

            let mut current_node_edges = Vec::new();

            if tile_position.x > 0 && tile_height > 0 {
                let top_node_idx = tilepos_to_idx(tile_position.x - 1, tile_position.y, length);
                let height_difference: i32 = height_map[top_node_idx] as i32 - tile_height as i32;
                if height_difference.abs() <= 1 {
                    current_node_edges.push(top_node_idx);
                    directed_graph_edges[top_node_idx].push(tile_idx);
                }
            }

            if tile_position.y > 0 && tile_height > 0 {
                let left_node_idx = tilepos_to_idx(tile_position.x, tile_position.y - 1, length);
                let height_difference: i32 = height_map[left_node_idx] as i32 - tile_height as i32;
                if height_difference.abs() <= 1 {
                    current_node_edges.push(left_node_idx);
                    directed_graph_edges[left_node_idx].push(tile_idx);
                }
            }

            directed_graph_edges.push(current_node_edges);
        }

        NodeEdges(directed_graph_edges)
    }

    /// Returns a Single Shortest Path between a source and target Tile
    /// Position, or nothing if none were found.
    pub fn shortest_path(&self, source: TilePos, target: TilePos, length: u32) -> Option<Path> {
        let graph_node_edges = &self.0;

        let mut node_distances = vec![0; graph_node_edges.len()];
        let mut node_parents = vec![-1; graph_node_edges.len()];
        let mut node_visited = vec![false; graph_node_edges.len()];

        let mapped_source_idx = tilepos_to_idx(source.x, source.y, length);
        let mut mapped_target_idx = tilepos_to_idx(target.x, target.y, length);

        let mut bfs_queue = VecDeque::from([mapped_source_idx]);
        while !bfs_queue.is_empty() {
            let current_node_idx = bfs_queue.pop_front().unwrap();
            if current_node_idx == mapped_target_idx {
                node_visited[current_node_idx] = true;
                break;
            }

            if node_visited[current_node_idx] {
                continue;
            }

            node_visited[current_node_idx] = true;

            for node_edge in &graph_node_edges[current_node_idx] {
                if node_visited[*node_edge] {
                    continue;
                }

                node_parents[*node_edge] = current_node_idx as i32;
                node_distances[*node_edge] = node_distances[current_node_idx] + 1;

                bfs_queue.push_back(*node_edge);
            }
        }

        if !node_visited[mapped_target_idx] {
            return None;
        }

        let mut path = VecDeque::from([mapped_target_idx]);
        while mapped_source_idx != mapped_target_idx {
            let node_parent = node_parents[mapped_target_idx];
            if node_parent == -1 {
                break;
            }

            mapped_target_idx = node_parent as usize;

            path.push_front(mapped_target_idx);
        }

        Some(Path(path))
    }
}

#[derive(Bundle)]
pub struct Graph {
    graph_type: GraphType,
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
    map_information: Query<(&TilemapGridSize, &TilemapType, &Transform)>,
    ground_graph_query: Query<(&NodeEdges, &NodeData, &GraphType)>,
    mut spawner: Commands,
) {
    if map_information.is_empty() {
        return;
    }

    let has_ground_graph = ground_graph_query
        .iter()
        .any(|graph_elements| graph_elements.2 == &GraphType::Ground);
    if has_ground_graph {
        return;
    }

    let heighted_tiles = tile_positions
        .iter()
        .map(|tile| HeightedTilePos::new(*tile.0, tile.1 .0 as u32))
        .collect::<Vec<HeightedTilePos>>();

    let layer_map_information = map_information
        .iter()
        .map(|layer_entry| (*layer_entry.0, *layer_entry.1, *layer_entry.2))
        .collect::<Vec<(TilemapGridSize, TilemapType, Transform)>>();

    let flipped_heighted_tiles = flip_heighted_tiles(&heighted_tiles);

    let node_data = NodeData::from_ground_tiles(&heighted_tiles, layer_map_information);
    let node_edges = NodeEdges::from_ground_tiles(flipped_heighted_tiles);

    spawner.spawn(Graph {
        graph_type: GraphType::Ground,
        node_data,
        node_edges,
    });
}

/// Spawns an Undirected Graph representing all water titles
pub fn create_water_graph(
    tile_positions: Query<(&TilePos, &LayerNumber)>,
    map_information: Query<(&TilemapSize, &TilemapGridSize, &TilemapType, &Transform)>,
    water_graph_query: Query<(&NodeEdges, &NodeData, &GraphType)>,
    mut spawner: Commands,
) {
    if map_information.is_empty() {
        return;
    }

    let has_water_graph = water_graph_query
        .iter()
        .any(|graph_elements| graph_elements.2 == &GraphType::Water);
    if has_water_graph {
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
                flip_y_axis_for_tile_pos(tile_pair.0.x, tile_pair.0.y, world_size),
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
        // connected to the water to be ignored. Think scenery
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

        let map_transform = map_information
            .iter()
            .nth(SUBSCRIBER_LAYER_NUM)
            .expect("Tile should be on this layer.")
            .3;

        let map_info = TiledMapInformation::new(grid_size, world_size, map_type, map_transform);
        let tiled_transform = tiled_to_bevy_transform(&tile_position, map_info);
        let node_data = tiled_transform.translation;
        let mut node_edges = Vec::new();

        if tile_position.x > 0 && tile_height == 0 {
            let top_node_idx = tilepos_to_idx(tile_position.x - 1, tile_position.y, world_size.x);
            let height_difference: i32 = height_map[top_node_idx] as i32 - tile_height as i32;
            if height_difference.abs() == 0 {
                node_edges.push(top_node_idx);
                directed_graph_edges[top_node_idx].push(tile_idx);
            }
        }

        if tile_position.y > 0 && tile_height == 0 {
            let left_node_idx = tilepos_to_idx(tile_position.x, tile_position.y - 1, world_size.x);
            let height_difference: i32 = height_map[left_node_idx] as i32 - tile_height as i32;
            if height_difference.abs() == 0 {
                node_edges.push(left_node_idx);
                directed_graph_edges[left_node_idx].push(tile_idx);
            }
        }

        directed_graph_data.push(node_data);
        directed_graph_edges.push(node_edges);
    }

    spawner.spawn(Graph {
        graph_type: GraphType::Water,
        node_data: NodeData(directed_graph_data),
        node_edges: NodeEdges(directed_graph_edges),
    });
}

/// Spawns an Undirected Graph representing all air titles
pub fn create_air_graph(
    tile_positions: Query<&TilePos>,
    map_information: Query<(&TilemapSize, &TilemapGridSize, &TilemapType, &Transform)>,
    air_graph_query: Query<(&NodeEdges, &NodeData, &GraphType)>,
    mut spawner: Commands,
) {
    if map_information.is_empty() {
        return;
    }

    let has_air_graph = air_graph_query
        .iter()
        .any(|graph_elements| graph_elements.2 == &GraphType::Air);
    if has_air_graph {
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

    // Sorting by Tile Position and Layer number ensures that we won't add
    // a previous node, whether above or to the left, that does not exist
    // yet.
    let mut tile_positions_sorted = tile_positions
        .iter()
        .map(|tile_pair| flip_y_axis_for_tile_pos(tile_pair.x, tile_pair.y, world_size))
        .collect::<Vec<TilePos>>();
    tile_positions_sorted.sort_by(|&pos1, &pos2| {
        let pos1_converted = tilepos_to_idx(pos1.x, pos1.y, world_size.x);
        let pos2_converted = tilepos_to_idx(pos2.x, pos2.y, world_size.x);

        pos1_converted.cmp(&pos2_converted)
    });

    // Perhaps it's a precaution, but given we are dealing with
    // many layers, and Tile Positions don't take into consideration
    // its layer (z value), we don't want to be dealing with
    // repeated entries.
    tile_positions_sorted.dedup();

    let mut directed_graph_data: Vec<Vec3> = Vec::new();
    let mut directed_graph_edges: Vec<Vec<usize>> = Vec::new();

    for tile_position in tile_positions_sorted {
        let tile_idx = tilepos_to_idx(tile_position.x, tile_position.y, world_size.x);

        let map_transform = map_information
            .iter()
            // The chatter is a Bird, so we want to consider the sky, which
            // happens to be the highest layer of tiles.
            .last()
            .expect("Tile should be on this layer.")
            .3;

        let map_info = TiledMapInformation::new(grid_size, world_size, map_type, map_transform);
        let tiled_transform = tiled_to_bevy_transform(&tile_position, map_info);
        let node_data = tiled_transform.translation;
        let mut node_edges = Vec::new();

        if tile_position.x > 0 {
            let left_node_idx = tilepos_to_idx(tile_position.x - 1, tile_position.y, world_size.x);
            node_edges.push(left_node_idx);
            directed_graph_edges[left_node_idx].push(tile_idx);
        }

        if tile_position.y > 0 {
            let top_node_idx = tilepos_to_idx(tile_position.x, tile_position.y - 1, world_size.x);
            node_edges.push(top_node_idx);
            directed_graph_edges[top_node_idx].push(tile_idx);
        }

        directed_graph_data.push(node_data);
        directed_graph_edges.push(node_edges);
    }

    spawner.spawn(Graph {
        graph_type: GraphType::Air,
        node_data: NodeData(directed_graph_data),
        node_edges: NodeEdges(directed_graph_edges),
    });
}

#[derive(Component, Deref, DerefMut)]
pub struct Target(pub Option<(Vec3, TilePos)>);

#[derive(Component)]
pub struct StartingPoint(pub Vec3, pub TilePos);

#[derive(Component, Deref, DerefMut)]
pub struct Path(pub VecDeque<usize>);

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

#[derive(Component, Deref, DerefMut)]
pub struct DestinationQueue(VecDeque<TilePos>);

#[derive(Component)]
pub struct SpawnPoint(pub TilePos);

#[derive(Bundle)]
pub struct PathInfo {
    spawn_pos: SpawnPoint,
    path: Path,
    requested_targets: DestinationQueue,
    start_pos: StartingPoint,
    target_pos: Target,
    direction: Direction,
    movement_timer: MovementTimer,
}

pub fn insert_pathing_information(
    moving_entities: Query<(Entity, &Transform, &TilePos), (With<MovementType>, Without<Path>)>,
    mut spawner: Commands,
) {
    for (moving_entity, entity_transform, entity_tilepos) in &moving_entities {
        spawner.entity(moving_entity).insert(PathInfo {
            spawn_pos: SpawnPoint(*entity_tilepos),
            path: Path(VecDeque::new()),
            requested_targets: DestinationQueue(VecDeque::new()),
            start_pos: StartingPoint(entity_transform.translation, *entity_tilepos),
            target_pos: Target(None),
            direction: Direction::TopRight,
            movement_timer: MovementTimer(Timer::from_seconds(0.05, TimerMode::Repeating)),
        });
    }
}

pub fn update_movement_target(
    mut moving_entity: Query<(
        &mut Target,
        &mut Path,
        &Transform,
        &mut Direction,
        &MovementType,
    )>,
    map_information: Query<&TilemapSize>,
    graph_query: Query<(&NodeData, &GraphType)>,
) {
    if graph_query.is_empty() {
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

    for (mut target, mut path, current_pos, mut direction, movement_type) in
        moving_entity.iter_mut()
    {
        let nodes = if *movement_type == MovementType::Walk {
            graph_query
                .iter()
                .find(|graph_elements| graph_elements.1 == &GraphType::Ground)
                .unwrap()
                .0
        } else if movement_type == &MovementType::Fly {
            graph_query
                .iter()
                .find(|graph_elements| graph_elements.1 == &GraphType::Air)
                .unwrap()
                .0
        } else {
            graph_query
                .iter()
                .find(|graph_elements| graph_elements.1 == &GraphType::Water)
                .unwrap()
                .0
        };

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
        &mut TilePos,
    )>,
    time: Res<Time>,
) {
    for (mut current_pos, mut target, mut movement_timer, mut starting_point, mut tile_pos) in
        &mut moving_entity
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
            *starting_point = StartingPoint(target_vec, target_tile_pos);
            *tile_pos = target_tile_pos;
            target.0 = None;
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

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct HeightedTilePos {
    xy: TilePos,
    z: u32,
}

impl PartialOrd for HeightedTilePos {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for HeightedTilePos {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.x()
            .cmp(&other.x())
            .then(self.y().cmp(&other.y()))
            .then(self.z().cmp(&other.z()))
    }
}

impl HeightedTilePos {
    pub fn new(tile_pos: TilePos, height: u32) -> Self {
        Self {
            xy: tile_pos,
            z: height,
        }
    }

    /// Returns a Tile Pos excluding the z value.
    pub fn truncate(&self) -> TilePos {
        self.xy
    }

    pub fn x(&self) -> u32 {
        self.xy.x
    }

    pub fn y(&self) -> u32 {
        self.xy.y
    }

    pub fn z(&self) -> u32 {
        self.z
    }

    /// Returns the position in Pixels (Using a Transform) of the Heighted
    /// Tile Position with respect to the size of the grid in pixels holding
    /// this tile.
    pub fn transform(&self, map_info: TiledMapInformation) -> Transform {
        let tile_pos = self.xy.clone();

        to_bevy_transform(&tile_pos, map_info)
    }

    /// Returns a new instance of a HeightedTilePos with its y axis flipped.
    pub fn flip(&self, width: u32) -> HeightedTilePos {
        let mapped_y = width - 1 - self.y();

        HeightedTilePos::new(TilePos::new(self.x(), mapped_y), self.z())
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

    enum IslandType {
        Square(u32, u32),
    }

    /// Returns a list of Tiles created to make an island, where
    /// an island consists of water, and land that is at least 1
    /// tile smaller to still show water.
    fn create_island(shape: IslandType) -> Vec<HeightedTilePos> {
        match shape {
            IslandType::Square(length, width) => {
                let mut tiles = Vec::new();

                tiles.append(&mut spawn_tiles_with_height(length, width, 0));
                tiles.append(&mut spawn_tiles_with_height(length / 2, width / 2, 1));

                tiles
            }
        }
    }

    /// Returns a rectangle or square of Tile Positions formed specified by the
    /// given length, width, and height.
    fn spawn_tiles_with_height(length: u32, width: u32, height: u32) -> Vec<HeightedTilePos> {
        let mut tiles = Vec::with_capacity(length as usize * width as usize * height as usize);
        for i in 0..length {
            for j in 0..width {
                tiles.push(HeightedTilePos::new(TilePos::new(i, j), height));
            }
        }

        tiles
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
        let expected_graph_type = GraphType::Ground;

        let mut graph_query = app.world.query::<(&NodeEdges, &GraphType)>();
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

        let mut graph_query = app.world.query::<(&NodeEdges, &GraphType)>();
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
        let expected_graph_type = GraphType::Ground;

        let mut graph_query = app.world.query::<(&NodeEdges, &GraphType)>();
        let (actual_node_edges, actual_graph_type) = graph_query.single(&app.world);

        assert_eq!(expected_node_edges, *actual_node_edges);
        assert_eq!(expected_graph_type, *actual_graph_type);
    }

    #[test]
    fn air_and_ground_graphs_different() {
        let layer = LayerNumber(SUBSCRIBER_LAYER_NUM);
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
        app.add_systems(Update, create_air_graph);
        app.update();

        let mut graph_query = app.world.query::<(&NodeEdges, &GraphType)>();
        let (ground_node_edges, ground_graph_type) = graph_query
            .iter(&app.world)
            .find(|graph_elements| graph_elements.1 == &GraphType::Ground)
            .expect("Ground Graph not created.");
        let (air_node_edges, air_graph_type) = graph_query
            .iter(&app.world)
            .find(|graph_elements| graph_elements.1 == &GraphType::Air)
            .expect("Air Graph not created.");

        assert_ne!(*ground_node_edges, *air_node_edges);
        assert_eq!(*ground_graph_type, GraphType::Ground);
        assert_eq!(*air_graph_type, GraphType::Air);
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
        let expected_graph_type = GraphType::Ground;

        let mut graph_query = app.world.query::<(&NodeEdges, &GraphType)>();
        let (actual_node_edges, actual_graph_type) = graph_query
            .iter(&app.world)
            .find(|graph_elements| graph_elements.1 == &GraphType::Ground)
            .unwrap();

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
        let expected_graph_type = GraphType::Ground;

        let mut graph_query = app.world.query::<(&NodeEdges, &GraphType)>();
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
        let expected_graph_type = GraphType::Ground;

        let mut graph_query = app.world.query::<(&NodeEdges, &GraphType)>();
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

    #[test]
    fn path_exists_for_source_and_target() {
        let square_island_tiles = create_island(IslandType::Square(4, 4));
        let (length, _width, _height) = dimensions_from(&square_island_tiles);

        let graph_edges = NodeEdges::from_ground_tiles(square_island_tiles);

        let source_pos = TilePos::new(0, 0);
        let target_pos = TilePos::new(0, 1);

        let path = graph_edges.shortest_path(source_pos, target_pos, length);

        assert!(path.is_some());
        assert_eq!(path.unwrap().len(), 2);
    }

    #[test]
    fn dimensions_from_heighted_tiles() {
        let square_island_tiles = create_island(IslandType::Square(4, 4));

        let (expected_length, expected_width, expected_height) = (4, 4, 2);
        let (actual_length, actual_width, actual_height) = dimensions_from(&square_island_tiles);

        assert_eq!(expected_length, actual_length);
        assert_eq!(expected_width, actual_width);
        assert_eq!(expected_height, actual_height);
    }

    #[test]
    fn height_map_from_heighted_tiles() {
        let square_island_tiles = create_island(IslandType::Square(4, 4));

        let expected_height_map = vec![1, 1, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let actual_height_map = height_map_from(&square_island_tiles);

        assert_eq!(expected_height_map, actual_height_map);
    }

    #[test]
    fn sorted_heighted_tiles() {
        let heighted_tiles = create_island(IslandType::Square(2, 2));

        let expected_sorted_tiles = vec![
            HeightedTilePos::new(TilePos::new(0, 0), 0),
            HeightedTilePos::new(TilePos::new(0, 0), 1),
            HeightedTilePos::new(TilePos::new(0, 1), 0),
            HeightedTilePos::new(TilePos::new(1, 0), 0),
            HeightedTilePos::new(TilePos::new(1, 1), 0),
        ];

        let mut actual_sorted_tiles = heighted_tiles.to_vec();
        actual_sorted_tiles.sort_unstable();

        assert_eq!(expected_sorted_tiles, actual_sorted_tiles);
    }

    #[test]
    fn unique_tiles_from_heighted_tiles() {
        let heighted_tiles = create_island(IslandType::Square(2, 2));

        let expected_tiles = vec![
            TilePos::new(0, 0),
            TilePos::new(0, 1),
            TilePos::new(1, 0),
            TilePos::new(1, 1),
        ];
        let actual_tiles = unique_tiles_from(&heighted_tiles);

        assert_eq!(expected_tiles, actual_tiles);
    }

    #[test]
    fn heighted_tiles_correctly_flipped() {
        let square_island_tiles = create_island(IslandType::Square(4, 4));
        let (_length, width, _height) = dimensions_from(&square_island_tiles);

        let expected_heighted_tiles = square_island_tiles
            .iter()
            .map(|tile| tile.flip(width))
            .collect::<Vec<HeightedTilePos>>();
        let actual_heighted_tiles = flip_heighted_tiles(&square_island_tiles);

        assert_eq!(expected_heighted_tiles, actual_heighted_tiles);
    }

    #[test]
    fn heighted_tile_can_flip() {
        let square_island_tiles = create_island(IslandType::Square(4, 4));
        let (_length, width, _height) = dimensions_from(&square_island_tiles);

        // Why 15? That's because the tile (3, 3) is found as the last entry,
        // and since the island is 4x4, and we start by zero, it's 4*4 - 1,
        // or 15.
        let tile_idx = 15;

        let heighted_tile = &square_island_tiles[tile_idx];

        let expected_flipped_tile = HeightedTilePos::new(TilePos::new(3, 0), 0);
        let actual_flipped_tile = heighted_tile.flip(width);

        assert_eq!(expected_flipped_tile, actual_flipped_tile);
    }
}
