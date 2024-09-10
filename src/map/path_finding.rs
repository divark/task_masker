use std::collections::{HashSet, VecDeque};

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::entities::{subscriber::SUBSCRIBER_LAYER_NUM, GameEntityType};

use super::tiled::{to_bevy_transform, LayerNumber, TiledMapInformation};

#[derive(Component, PartialEq, Debug)]
pub enum GraphType {
    Ground,
    Air,
    Water,
}

pub struct TranslationGatherer {
    map_information: Vec<TileLayerPosition>,
}

impl TranslationGatherer {
    pub fn new(map_information: Vec<TileLayerPosition>) -> Self {
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

            let tile_layer_position = self
                .map_information
                .get(tile_height)
                .expect("translations_from: Could not find map information at given tile height.");

            let grid_size = tile_layer_position.get_grid_size();
            let map_type = tile_layer_position.get_map_type();
            let map_position = tile_layer_position.get_position();

            let tile_translation = tile
                .center_in_world(grid_size, map_type)
                .extend(map_position.translation.z);
            let tile_transform = *map_position * Transform::from_translation(tile_translation);

            heighted_tile_translations.push(tile_transform.translation);
        }

        heighted_tile_translations
    }

    /// Returns the Highest Transform (Position) found for each Heighted Tile.
    pub fn highest_translations_from(&self, heighted_tiles: &Vec<HeightedTilePos>) -> Vec<Vec3> {
        let mut heighted_tile_translations = Vec::new();

        let unique_tiles = unique_tiles_from(heighted_tiles);
        for tile in unique_tiles {
            let tile_layer_position = self.map_information.iter().last().expect(
                "highest_translations_from: Could not find map information at the highest
                tile height.",
            );

            let grid_size = tile_layer_position.get_grid_size();
            let map_type = tile_layer_position.get_map_type();
            let map_position = tile_layer_position.get_position();

            let tile_translation = tile
                .center_in_world(grid_size, map_type)
                .extend(map_position.translation.z);
            let tile_transform = *map_position * Transform::from_translation(tile_translation);

            heighted_tile_translations.push(tile_transform.translation);
        }

        heighted_tile_translations
    }

    /// Returns the Lowest Transform (Position) found for each Heighted Tile.
    pub fn translations_at_height(
        &self,
        heighted_tiles: &Vec<HeightedTilePos>,
        height: usize,
    ) -> Vec<Vec3> {
        let mut heighted_tile_translations = Vec::new();

        let unique_tiles = unique_tiles_from(heighted_tiles);
        for tile in unique_tiles {
            let tile_layer_position = self.map_information.get(height).expect(
                "lowest_translations_from: Could not find map information at the specified
                tile height.",
            );

            let grid_size = tile_layer_position.get_grid_size();
            let map_type = tile_layer_position.get_map_type();
            let map_position = tile_layer_position.get_position();

            let tile_translation = tile
                .center_in_world(grid_size, map_type)
                .extend(map_position.translation.z);
            let tile_transform = *map_position * Transform::from_translation(tile_translation);

            heighted_tile_translations.push(tile_transform.translation);
        }

        heighted_tile_translations
    }
}

#[derive(Component)]
pub struct UndirectedGraph {
    tile_type: GraphType,
    length: u32,
    nodes: NodeData,
    edges: NodeEdges,
}

impl UndirectedGraph {
    /// Converts a Tile map with layers into an
    /// Undirected Graph depending on the type
    /// of Tiles being considered.
    pub fn from_tiles(
        tile_type: GraphType,
        tiles: Vec<HeightedTilePos>,
        tile_layers: Vec<TileLayerPosition>,
    ) -> Self {
        let (length, _width, _height) = dimensions_from(&tiles);
        let (nodes, edges) = match tile_type {
            GraphType::Ground => (
                NodeData::from_ground_tiles(&tiles, tile_layers),
                NodeEdges::from_ground_tiles(tiles),
            ),
            GraphType::Air => (
                NodeData::from_air_tiles(&tiles, tile_layers),
                NodeEdges::from_air_tiles(tiles),
            ),
            GraphType::Water => (
                NodeData::from_water_tiles(&tiles, tile_layers),
                NodeEdges::from_water_tiles(tiles),
            ),
        };

        Self {
            tile_type,
            length,
            nodes,
            edges,
        }
    }

    /// Returns a Shortest Path for some start and destination
    /// Tile Positions.
    pub fn shortest_path(&self, start: TilePos, end: TilePos) -> Option<Path> {
        self.edges.shortest_path(start, end, self.length)
    }

    /// Returns the contents of a Node found in the
    /// Undirected Graph.
    pub fn get_node(&self, index: usize) -> Option<&Vec3> {
        self.nodes.0.get(index)
    }

    /// Returns what type of Nodes are being held in
    /// the Undirected Graph.
    pub fn get_node_type(&self) -> &GraphType {
        &self.tile_type
    }

    /// Returns a reference to the edges recorded in the
    /// undirected graph.
    pub fn edges(&self) -> &NodeEdges {
        &self.edges
    }
}

#[derive(Component, Clone)]
pub struct NodeData(pub Vec<Vec3>);

#[derive(Clone)]
pub struct TileLayerPosition {
    grid_size: TilemapGridSize,
    map_type: TilemapType,
    position: Transform,
}

impl TileLayerPosition {
    pub fn new(grid_size: TilemapGridSize, map_type: TilemapType, position: Transform) -> Self {
        Self {
            grid_size,
            map_type,
            position,
        }
    }

    pub fn get_grid_size(&self) -> &TilemapGridSize {
        &self.grid_size
    }

    pub fn get_map_type(&self) -> &TilemapType {
        &self.map_type
    }

    pub fn get_position(&self) -> &Transform {
        &self.position
    }
}

impl NodeData {
    pub fn from_ground_tiles(
        heighted_tiles: &Vec<HeightedTilePos>,
        layer_map_information: Vec<TileLayerPosition>,
    ) -> Self {
        let translation_gatherer = TranslationGatherer::new(layer_map_information);

        let tile_translations = translation_gatherer.translations_from(heighted_tiles);

        NodeData(tile_translations)
    }

    pub fn from_air_tiles(
        heighted_tiles: &Vec<HeightedTilePos>,
        layer_map_information: Vec<TileLayerPosition>,
    ) -> Self {
        let translation_gatherer = TranslationGatherer::new(layer_map_information);

        let tile_translations = translation_gatherer.highest_translations_from(heighted_tiles);

        NodeData(tile_translations)
    }

    pub fn from_water_tiles(
        heighted_tiles: &Vec<HeightedTilePos>,
        layer_map_information: Vec<TileLayerPosition>,
    ) -> Self {
        let translation_gatherer = TranslationGatherer::new(layer_map_information);

        let tile_translations =
            translation_gatherer.translations_at_height(heighted_tiles, SUBSCRIBER_LAYER_NUM);

        NodeData(tile_translations)
    }
}

#[derive(Component, Clone, PartialEq, Debug)]
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

    /// Returns a set of Node Edges derived from a collection of Tiles
    /// designated for Air traversal.
    pub fn from_air_tiles(air_tiles: Vec<HeightedTilePos>) -> NodeEdges {
        let mut directed_graph_edges: Vec<Vec<usize>> = Vec::with_capacity(air_tiles.len());

        let (length, _width, _height) = dimensions_from(&air_tiles);

        let tile_positions_no_layers = unique_tiles_from(&air_tiles);
        for tile_position in tile_positions_no_layers {
            let tile_idx = tilepos_to_idx(tile_position.x, tile_position.y, length);

            let mut current_node_edges = Vec::new();
            if tile_position.x > 0 {
                let top_node_idx = tilepos_to_idx(tile_position.x - 1, tile_position.y, length);

                current_node_edges.push(top_node_idx);
                directed_graph_edges[top_node_idx].push(tile_idx);
            }

            if tile_position.y > 0 {
                let left_node_idx = tilepos_to_idx(tile_position.x, tile_position.y - 1, length);
                current_node_edges.push(left_node_idx);
                directed_graph_edges[left_node_idx].push(tile_idx);
            }

            directed_graph_edges.push(current_node_edges);
        }

        NodeEdges(directed_graph_edges)
    }

    /// Returns a set of Node Edges derived from a collection of Tiles
    /// designated for Water traversal.
    pub fn from_water_tiles(water_tiles: Vec<HeightedTilePos>) -> NodeEdges {
        let mut directed_graph_edges: Vec<Vec<usize>> = Vec::with_capacity(water_tiles.len());

        let height_map: Vec<usize> = height_map_from(&water_tiles);
        let (length, _width, _height) = dimensions_from(&water_tiles);

        let tile_positions_no_layers = unique_tiles_from(&water_tiles);

        for tile_position in tile_positions_no_layers {
            let tile_idx = tilepos_to_idx(tile_position.x, tile_position.y, length);
            let tile_height = height_map[tile_idx];

            let mut current_node_edges = Vec::new();

            if tile_position.x > 0 && tile_height == 0 {
                let top_node_idx = tilepos_to_idx(tile_position.x - 1, tile_position.y, length);
                let height_difference: i32 = height_map[top_node_idx] as i32 - tile_height as i32;
                if height_difference.abs() == 0 {
                    current_node_edges.push(top_node_idx);
                    directed_graph_edges[top_node_idx].push(tile_idx);
                }
            }

            if tile_position.y > 0 && tile_height == 0 {
                let left_node_idx = tilepos_to_idx(tile_position.x, tile_position.y - 1, length);
                let height_difference: i32 = height_map[left_node_idx] as i32 - tile_height as i32;
                if height_difference.abs() == 0 {
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
        let source_tilepos = source; //convert_tiled_to_bevy_pos(source, length);
        let target_tilepos = target; //convert_tiled_to_bevy_pos(target, length);

        let graph_node_edges = &self.0;

        let mut node_distances = vec![0; graph_node_edges.len()];
        let mut node_parents = vec![-1; graph_node_edges.len()];
        let mut node_visited = vec![false; graph_node_edges.len()];

        let mapped_source_idx = tilepos_to_idx(source_tilepos.x, source_tilepos.y, length);
        let mut mapped_target_idx = tilepos_to_idx(target_tilepos.x, target_tilepos.y, length);

        let mut bfs_queue = VecDeque::from([mapped_source_idx]);
        while !bfs_queue.is_empty() {
            let current_node_idx = bfs_queue
                .pop_front()
                .expect("shortest_path: BFS Queue is Empty, but it shouldn't be?");
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
    ground_graph_query: Query<&UndirectedGraph>,
    mut spawner: Commands,
) {
    if map_information.is_empty() {
        return;
    }

    let has_ground_graph = ground_graph_query
        .iter()
        .any(|graph| *graph.get_node_type() == GraphType::Ground);
    if has_ground_graph {
        return;
    }

    let heighted_tiles = tile_positions
        .iter()
        .map(|tile| HeightedTilePos::new(*tile.0, tile.1 .0 as u32))
        .collect::<Vec<HeightedTilePos>>();

    let layer_map_information = map_information
        .iter()
        .map(|layer_entry| TileLayerPosition::new(*layer_entry.0, *layer_entry.1, *layer_entry.2))
        .collect::<Vec<TileLayerPosition>>();

    let node_data = NodeData::from_ground_tiles(&heighted_tiles, layer_map_information.clone());
    let node_edges = NodeEdges::from_ground_tiles(heighted_tiles.clone());

    spawner.spawn(UndirectedGraph::from_tiles(
        GraphType::Ground,
        heighted_tiles,
        layer_map_information,
    ));

    spawner.spawn(Graph {
        graph_type: GraphType::Ground,
        node_data,
        node_edges,
    });
}

/// Spawns an Undirected Graph representing all water titles
pub fn create_water_graph(
    tile_positions: Query<(&TilePos, &LayerNumber)>,
    map_information: Query<(&TilemapGridSize, &TilemapType, &Transform)>,
    water_graph_query: Query<&UndirectedGraph>,
    mut spawner: Commands,
) {
    if map_information.is_empty() {
        return;
    }

    let has_water_graph = water_graph_query
        .iter()
        .any(|graph| *graph.get_node_type() == GraphType::Water);
    if has_water_graph {
        return;
    }

    let heighted_tiles = tile_positions
        .iter()
        .map(|tile| HeightedTilePos::new(*tile.0, tile.1 .0 as u32))
        .collect::<Vec<HeightedTilePos>>();

    let layer_map_information = map_information
        .iter()
        .map(|layer_entry| TileLayerPosition::new(*layer_entry.0, *layer_entry.1, *layer_entry.2))
        .collect::<Vec<TileLayerPosition>>();

    let node_data = NodeData::from_water_tiles(&heighted_tiles, layer_map_information.clone());
    let node_edges = NodeEdges::from_water_tiles(heighted_tiles.clone());

    spawner.spawn(UndirectedGraph::from_tiles(
        GraphType::Water,
        heighted_tiles,
        layer_map_information,
    ));

    spawner.spawn(Graph {
        graph_type: GraphType::Water,
        node_data,
        node_edges,
    });
}

/// Spawns an Undirected Graph representing all air titles
pub fn create_air_graph(
    tile_positions: Query<(&TilePos, &LayerNumber)>,
    map_information: Query<(&TilemapGridSize, &TilemapType, &Transform)>,
    air_graph_query: Query<&UndirectedGraph>,
    mut spawner: Commands,
) {
    if map_information.is_empty() {
        return;
    }

    let has_air_graph = air_graph_query
        .iter()
        .any(|graph| *graph.get_node_type() == GraphType::Air);
    if has_air_graph {
        return;
    }

    let heighted_tiles = tile_positions
        .iter()
        .map(|tile| HeightedTilePos::new(*tile.0, tile.1 .0 as u32))
        .collect::<Vec<HeightedTilePos>>();

    let layer_map_information = map_information
        .iter()
        .map(|layer_entry| TileLayerPosition::new(*layer_entry.0, *layer_entry.1, *layer_entry.2))
        .collect::<Vec<TileLayerPosition>>();

    spawner.spawn(UndirectedGraph::from_tiles(
        GraphType::Air,
        heighted_tiles,
        layer_map_information,
    ));
}

#[derive(Component, Deref, DerefMut)]
pub struct Target(pub Option<(Vec3, TilePos)>);

#[derive(Component)]
pub struct StartingPoint(pub Vec3, pub TilePos);

#[derive(Component, Deref, DerefMut)]
pub struct Path(pub VecDeque<usize>);

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
    moving_entities: Query<(Entity, &Transform, &TilePos), (With<GameEntityType>, Without<Path>)>,
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
        &GameEntityType,
    )>,
    map_information: Query<&TilemapSize>,
    graph_query: Query<&UndirectedGraph>,
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
        let tile_graph = if *movement_type == GameEntityType::Walk {
            graph_query
                .iter()
                .find(|graph| graph.get_node_type() == &GraphType::Ground)
                .expect("update_movement_target: Could not find Ground Graph")
        } else if movement_type == &GameEntityType::Fly {
            graph_query
                .iter()
                .find(|graph| graph.get_node_type() == &GraphType::Air)
                .expect("update_movement_target: Could not find Air Graph")
        } else {
            graph_query
                .iter()
                .find(|graph| graph.get_node_type() == &GraphType::Water)
                .expect("update_movement_target: Could not find Water Graph")
        };

        if target.0.is_some() || path.0.is_empty() {
            continue;
        }

        let new_target = path
            .0
            .pop_front()
            .expect("The path was not supposed to be empty by here.");
        let target_tile_pos = idx_to_tilepos(new_target, world_size.y);
        let target_pos = tile_graph.get_node(new_target).unwrap();

        if let Some(new_direction) =
            get_direction(*current_pos, Transform::from_translation(*target_pos))
        {
            *direction = new_direction;
        }

        target.0 = Some((*target_pos, target_tile_pos));
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
        let tile_pos = self.xy;

        to_bevy_transform(&tile_pos, map_info)
    }

    /// Returns a new instance of a HeightedTilePos with its y axis flipped.
    pub fn flip(&self, width: u32) -> HeightedTilePos {
        let mapped_y = width - 1 - self.y();

        HeightedTilePos::new(TilePos::new(self.x(), mapped_y), self.z())
    }
}
