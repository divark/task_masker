use std::time::Duration;

use bevy::prelude::*;

use bevy_ecs_tilemap::prelude::*;
use task_masker::entities::{chatter::*, crop::*, fruit::*, streamer::*, subscriber::*};
use task_masker::map::path_finding::{
    tilepos_to_idx, unique_tiles_from, GraphType, HeightedTilePos, MovementTimer, NodeData,
    NodeEdges, Path, Target,
};
use task_masker::map::plugins::{PathFindingPlugin, TilePosEvent};
use task_masker::map::tiled::{
    flip_y_axis_for_tile_pos, spawn_tiles_from_tiledmap, to_bevy_transform, LayerNumber,
    TiledMapInformation,
};
use task_masker::GameState;

#[derive(Default)]
pub struct MockTiledMapPlugin;

impl Plugin for MockTiledMapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_tiles_from_tiledmap);
    }
}

#[derive(Default)]
pub struct MockStreamerPlugin;

impl Plugin for MockStreamerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                spawn_player_tile,
                move_streamer,
                queue_destination_for_streamer,
                update_status_when_speaking,
            ),
        );
    }
}

#[derive(Default)]
pub struct MockSubscriberPlugin;

impl Plugin for MockSubscriberPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SubscriberMsg>();
        app.add_systems(
            Update,
            (
                mock_replace_subscriber,
                //trigger_swimming_to_streamer,
                swim_to_streamer_to_speak,
                leave_from_streamer_from_subscriber,
                return_subscriber_to_idle,
                follow_streamer_while_approaching_for_subscriber,
            ),
        );
    }
}

#[derive(Default)]
pub struct MockChatterPlugin;

impl Plugin for MockChatterPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ChatMsg>();
        app.add_systems(
            Update,
            (
                replace_chatter_tile,
                //trigger_flying_to_streamer,
                fly_to_streamer_to_speak,
                leave_from_streamer_from_chatter,
                return_chatter_to_idle,
                follow_streamer_while_speaking,
                follow_streamer_while_approaching_for_chatter,
            ),
        );
    }
}

#[derive(Default)]
pub struct MockFruitPlugin;

impl Plugin for MockFruitPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                replace_fruit_tiles,
                make_fruit_fall,
                make_fruit_dropped,
                pathfind_streamer_to_fruit,
                claim_fruit_from_streamer,
                respawn_fruit,
            ),
        );
    }
}

#[derive(Default)]
pub struct MockCropPlugin;

impl Plugin for MockCropPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<NewSubscriber>();
        app.add_systems(
            Update,
            (
                replace_crop_tiles,
                grow_crops,
                pathfind_streamer_to_crops,
                pick_up_crops,
            ),
        );
    }
}

#[derive(PartialEq)]
enum EntityType {
    Streamer,
    Chatter,
    Subscriber,
    Fruit,
    Crop,
    Tile,
}

fn intercept_movement_timer(mut timer_query: Query<&mut MovementTimer, Added<MovementTimer>>) {
    for mut movement_timer in &mut timer_query {
        movement_timer.0 = Timer::new(Duration::from_secs(0), TimerMode::Repeating);
    }
}

/// A convenience abstraction of Task Masker
/// represented as what is needed to create
/// one of its worlds.
struct GameWorld {
    pub app: App,

    map_size: TilemapSize,
}

impl GameWorld {
    pub fn new() -> Self {
        let mut app = App::new();

        app.init_state::<GameState>();
        app.insert_state(GameState::InGame);
        app.add_plugins(MinimalPlugins);

        app.add_plugins(MockTiledMapPlugin);
        app.add_plugins(MockStreamerPlugin);
        app.add_plugins(MockSubscriberPlugin);
        app.add_plugins(MockChatterPlugin);
        // TODO: Split generating sound effects for each change
        // in state for both Fruit and Crops.
        //app.add_plugins(MockFruitPlugin);
        //app.add_plugins(MockCropPlugin);
        app.add_plugins(PathFindingPlugin);

        app.add_systems(Update, intercept_movement_timer);

        app.update();

        let map_size = app
            .world
            .query::<&TilemapSize>()
            .iter(&app.world)
            .next()
            .unwrap()
            .clone();

        Self { app, map_size }
    }

    /// Returns a reference to the Entity just created
    /// based on its type.
    pub fn find(&mut self, entity_type: EntityType) -> Entity {
        self.app.update();

        match entity_type {
            EntityType::Streamer => {
                return self
                    .app
                    .world
                    .query::<(Entity, &StreamerLabel)>()
                    .get_single(&self.app.world)
                    .expect("spawn: Streamer was not found after trying to spawn it.")
                    .0;
            }
            EntityType::Subscriber => {
                return self
                    .app
                    .world
                    .query::<(Entity, &SubscriberLabel)>()
                    .get_single(&self.app.world)
                    .expect("spawn: Subscriber was not found after trying to spawn it.")
                    .0;
            }
            EntityType::Chatter => {
                return self
                    .app
                    .world
                    .query::<(Entity, &ChatterLabel)>()
                    .get_single(&self.app.world)
                    .expect("spawn: Chatter was not found after trying to spawn it.")
                    .0;
            }
            _ => todo!(),
        }
    }

    /// Returns a reference to the Entity just created
    /// based on its type and location.
    pub fn find_at(&mut self, entity_type: EntityType, position: HeightedTilePos) -> Entity {
        match entity_type {
            EntityType::Fruit => {
                self.app.update();

                return self
                    .app
                    .world
                    .query_filtered::<(Entity, &TilePos), With<FruitState>>()
                    .iter(&self.app.world)
                    .find(|fruit_entry| *fruit_entry.1 == position.truncate())
                    .expect("find: Fruit was not found after trying to spawn it.")
                    .0;
            }
            EntityType::Crop => {
                self.app.update();

                return self
                    .app
                    .world
                    .query_filtered::<(Entity, &TilePos), With<CropState>>()
                    .iter(&self.app.world)
                    .find(|crop_entry| *crop_entry.1 == position.truncate())
                    .expect("find: Crop was not found after trying to spawn it.")
                    .0;
            }
            _ => todo!(),
        }
    }

    /// Returns a reference to the Tile Entity found
    /// at the desired position.
    pub fn tile_at_position(&mut self, tile_pos: TilePos, height: u32) -> Entity {
        let map_size = self
            .app
            .world
            .query::<&TilemapSize>()
            .iter(&self.app.world)
            .nth(height as usize)
            .expect("tile_at_position: Could not get map size given the specified height.")
            .clone();

        let flipped_tile_pos = flip_y_axis_for_tile_pos(tile_pos.x, tile_pos.y, &map_size);
        self.app
            .world
            .query::<(Entity, &TilePos, &LayerNumber)>()
            .iter(&self.app.world)
            .find(|tile_entry| {
                *tile_entry.1 == flipped_tile_pos && tile_entry.2 .0 == height as usize
            })
            .map(|tile_entry| tile_entry.0)
            .expect("tile_at_position: Could not find Tile at given Tile Pos and height.")
    }

    /// Returns the Height represented as the Z value
    /// for some given Entity.
    pub fn height_of(&mut self, entity: Entity) -> f32 {
        // Did you know that Tiles by default do not have
        // a Transform? Because of that, we have to interpret
        // height based on its Layer Number.
        let found_height = if self.get_entity_type(entity) == EntityType::Tile {
            self.app
                .world
                .query::<(&TilePos, &LayerNumber)>()
                .get(&self.app.world, entity)
                .unwrap()
                .1
                // A Layer Number is only equivalent to a Translation's z value
                // when it is doubled after some observation in a debugger.
                 .0 as f32
                * 2.0
        } else {
            self.app
                .world
                .query::<&Transform>()
                .get(&self.app.world, entity)
                .unwrap()
                .translation
                .z
        };

        found_height
    }

    fn get_path_from(&mut self, source_entity: Entity, target_pos: TilePos) -> &Path {
        let source_path = match self.get_entity_type(source_entity) {
            EntityType::Streamer => {
                self.app
                    .world
                    .send_event(TilePosEvent::new(target_pos))
                    .unwrap();

                self.app.update();
                self.app.update();
                self.app
                    .world
                    .query::<&Path>()
                    .get(&self.app.world, source_entity)
                    .expect("travel_to: Path for Streamer not populated yet.")
            }
            EntityType::Chatter => {
                self.app.world.send_event(ChatMsg {
                    name: "Chatter".to_string(),
                    msg: "Hello Caveman!".to_string(),
                });

                self.app.update();
                self.app.update();
                self.app
                    .world
                    .query::<&Path>()
                    .get(&self.app.world, source_entity)
                    .expect("travel_to: Path for Streamer not populated yet.")
            }
            EntityType::Subscriber => {
                self.app.world.send_event(SubscriberMsg {
                    name: String::from("Subscriber"),
                    msg: String::from("'Ello Caveman!"),
                });

                self.app.update();
                self.app.update();
                self.app
                    .world
                    .query::<&Path>()
                    .get(&self.app.world, source_entity)
                    .expect("travel_to: Path for Streamer not populated yet.")
            }
            _ => panic!("travel_to: Incompatiable Entity passed for Traveling."),
        };

        source_path
    }

    /// Triggers the Source Entity to move to the location
    /// of the Target Entity.
    pub fn travel_to(&mut self, source_entity: Entity, target_entity: Entity) {
        let target_pos = self.get_tile_pos_from(target_entity);

        while self.get_path_from(source_entity, target_pos).len() > 0 {
            self.app.update();
        }

        loop {
            self.app.update();

            let target = self.app.world.get::<Target>(source_entity).unwrap();
            if target.is_none() {
                break;
            }
        }
    }

    /// Returns a boolean representing whether two Entities
    /// co-exist in the same location.
    pub fn has_reached_tile(&mut self, source_entity: Entity, target_entity: Entity) {
        let source_pos = self.get_tile_pos_from(source_entity);
        let target_pos = self.get_tile_pos_from(target_entity);
        match self.get_entity_type(source_entity) {
            EntityType::Subscriber => assert!(self.next_to_land(source_pos)),
            EntityType::Chatter => {
                assert_eq!(distance_of(source_pos, target_pos), DIST_AWAY_FROM_STREAMER)
            }
            _ => assert_eq!(distance_of(source_pos, target_pos), 0),
        }
    }

    /// Returns the Transform for some given Entity.
    fn get_transform_from(&mut self, entity: Entity) -> Vec2 {
        let entity_type = self.get_entity_type(entity);
        if entity_type == EntityType::Tile {
            panic!("get_transform_from: Tile given for get_transform_from. Call get_tile_transform_from instead.")
        }

        self.app
            .world
            .get::<Transform>(entity)
            .expect("get_transform_from: Could not find Transform for given Entity.")
            .translation
            .truncate()
    }

    /// Returns the Transform for some given Tile Entity.
    pub fn get_tile_transform_from(&mut self, entity: Entity, tile_height: usize) -> Vec2 {
        let tile_pos = self.get_tile_pos_from(entity);
        return to_bevy_transform(&tile_pos, self.map_info(tile_height))
            .translation
            .truncate();
    }

    /// Asserts whether two Entities co-exist in the same location
    /// at the Transform level.
    pub fn has_reached_transform(&mut self, source_entity: Entity, target_entity: Entity) {
        let source_transform = self.get_transform_from(source_entity);
        let target_transform = self.get_transform_from(target_entity);

        assert_eq!(source_transform, target_transform);
    }

    /// Asserts whether two Tile Entities co-exist in the same location
    /// at the Transform level.
    pub fn has_reached_tile_transform(
        &mut self,
        source_entity: Entity,
        target_entity: Entity,
        source_height: usize,
        target_height: usize,
    ) {
        let source_tile_transform = self.get_tile_transform_from(source_entity, source_height);
        let target_tile_transform = self.get_tile_transform_from(target_entity, target_height);

        assert_eq!(source_tile_transform, target_tile_transform);
    }

    /// Returns true when the source position has any Ground Tile neighboring
    /// it, where neighbors are left-to-right, or top-to-bottom only.
    fn next_to_land(&mut self, source_pos: TilePos) -> bool {
        let neighbors = self.get_tile_neighbors(&source_pos);

        let ground_nodes = self
            .app
            .world
            .query::<(&NodeEdges, &GraphType)>()
            .iter(&self.app.world)
            .find(|entry| *entry.1 == GraphType::Ground)
            .map(|entry| entry.0)
            .expect(
                "next_to_land: Ground Nodes were expected to be loaded, but they were not found",
            );

        let mut next_to_land = false;
        for neighbor in neighbors {
            next_to_land = !ground_nodes.0[neighbor].is_empty();
            if next_to_land {
                break;
            }
        }

        next_to_land
    }

    /// Returns the Tiles neighboring the source position.
    fn get_tile_neighbors(&mut self, source_pos: &TilePos) -> Vec<usize> {
        let world_size = self
            .app
            .world
            .query::<&TilemapSize>()
            .iter(&self.app.world)
            .max_by(|&x, &y| {
                let x_world_area = x.x * x.y;
                let y_world_area = y.x * y.y;

                x_world_area.cmp(&y_world_area)
            })
            .expect("get_tile_neighbors: Could not find largest world size. Is the map loaded?");

        let left_tilepos = TilePos::new(source_pos.x - 1, source_pos.y);
        let top_tilepos = TilePos::new(source_pos.x, source_pos.y + 1);
        let right_tilepos = TilePos::new(source_pos.x + 1, source_pos.y);
        let bottom_tilepos = TilePos::new(source_pos.x, source_pos.y - 1);

        vec![left_tilepos, top_tilepos, right_tilepos, bottom_tilepos]
            .iter_mut()
            .map(|tile_pos| tilepos_to_idx(tile_pos.x, tile_pos.y, world_size.x))
            .collect::<Vec<usize>>()
    }

    /// Returns the Tiled TilePos for some given Entity.
    fn get_tile_pos_from(&mut self, entity: Entity) -> TilePos {
        let entity_type = self.get_entity_type(entity);

        let found_tile = self
            .app
            .world
            .query::<&TilePos>()
            .get(&self.app.world, entity)
            .unwrap()
            .clone();

        if entity_type != EntityType::Tile {
            return found_tile;
        }

        flip_y_axis_for_tile_pos(found_tile.x, found_tile.y, &self.map_size)
    }

    /// Returns an EntityType based on what was found for the
    /// given entity.
    fn get_entity_type(&mut self, entity: Entity) -> EntityType {
        let is_streamer = self
            .app
            .world
            .query::<&StreamerLabel>()
            .get(&self.app.world, entity)
            .is_ok();

        if is_streamer {
            return EntityType::Streamer;
        }

        let is_subscriber = self
            .app
            .world
            .query::<&SubscriberLabel>()
            .get(&self.app.world, entity)
            .is_ok();

        if is_subscriber {
            return EntityType::Subscriber;
        }

        let is_chatter = self
            .app
            .world
            .query::<&ChatterLabel>()
            .get(&self.app.world, entity)
            .is_ok();

        if is_chatter {
            return EntityType::Chatter;
        }

        return EntityType::Tile;
    }

    /// Returns whether a Tiled-based Tile Position exists in
    /// a populated Graph based on its type.
    pub fn tile_has_neighbors(&mut self, graph_type: GraphType, tiled_tile_pos: TilePos) -> bool {
        let bevy_tile_pos =
            flip_y_axis_for_tile_pos(tiled_tile_pos.x, tiled_tile_pos.y, &self.map_size);

        let node_edges = self
            .app
            .world
            .query::<(&NodeEdges, &GraphType)>()
            .iter(&self.app.world)
            .filter(|graph_entry| *graph_entry.1 == graph_type)
            .map(|graph_entry| graph_entry.0)
            .next()
            .unwrap();

        //.find(|graph_entry| {
        let node_idx = tilepos_to_idx(bevy_tile_pos.x, bevy_tile_pos.y, self.map_size.x);

        let graph_edges = &node_edges.0;
        let is_not_isolated = graph_edges[node_idx].len() > 0;

        is_not_isolated
        //})
        //.is_some()
    }

    /// Returns Map Measurement Information based on the specified
    /// Tile Layer Height.
    pub fn map_info(&mut self, height: usize) -> TiledMapInformation {
        let (grid_size, map_size, map_type, map_transform) = self
            .app
            .world
            .query::<(&TilemapGridSize, &TilemapSize, &TilemapType, &Transform)>()
            .iter(&self.app.world)
            .nth(height)
            .expect("map_info: Could not generate TiledMapInformation since all required components are missing.");

        TiledMapInformation::new(grid_size, map_size, map_type, map_transform)
    }
}

/// Returns the approximate number of Tiles away the target_pos
/// is from source_pos
fn distance_of(source_pos: TilePos, target_pos: TilePos) -> usize {
    let x1 = source_pos.x as f32;
    let x2 = target_pos.x as f32;

    let y1 = source_pos.y as f32;
    let y2 = target_pos.y as f32;

    ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt().floor() as usize
}

#[test]
fn creating_gameworld_does_not_crash() {
    let mut world = GameWorld::new();

    assert_ne!(
        world
            .app
            .world
            .query::<&TilePos>()
            .iter(&world.app.world)
            .len(),
        0
    );
}

#[test]
fn chatter_higher_than_streamer() {
    let mut world = GameWorld::new();

    let streamer = world.find(EntityType::Streamer);
    let chatter = world.find(EntityType::Chatter);

    assert!(world.height_of(chatter) > world.height_of(streamer));
}

#[test]
fn chatter_spawned_at_right_tilepos() {
    let mut world = GameWorld::new();

    let chatter = world.find(EntityType::Chatter);

    let expected_tilepos = TilePos::new(69, 20);
    let chatter_tilepos = world.get_tile_pos_from(chatter);

    assert_eq!(expected_tilepos, chatter_tilepos);
}

#[test]
fn streamer_spawned_at_right_tilepos() {
    let mut world = GameWorld::new();

    let streamer = world.find(EntityType::Streamer);

    let expected_tilepos = TilePos::new(42, 59);
    let streamer_tilepos = world.get_tile_pos_from(streamer);

    assert_eq!(expected_tilepos, streamer_tilepos);
}

#[test]
fn tile_spawned_at_right_tilepos() {
    let mut world = GameWorld::new();

    let expected_tilepos = TilePos::new(46, 58);
    let expected_tile_height = 6;

    let tile = world.tile_at_position(expected_tilepos, expected_tile_height);

    let actual_tilepos = world.get_tile_pos_from(tile);

    assert_eq!(expected_tilepos, actual_tilepos);
}

#[test]
fn subscriber_lower_than_streamer() {
    let mut world = GameWorld::new();

    let streamer = world.find(EntityType::Streamer);
    let subscriber = world.find(EntityType::Subscriber);

    assert!(world.height_of(subscriber) < world.height_of(streamer));
}

#[test]
fn streamer_and_subscriber_far_away_by_default() {
    let mut world = GameWorld::new();

    let streamer = world.find(EntityType::Streamer);
    let subscriber = world.find(EntityType::Subscriber);

    let streamer_pos = world.get_tile_pos_from(streamer);
    let subscriber_pos = world.get_tile_pos_from(subscriber);

    assert_ne!(streamer_pos, subscriber_pos);
    assert!(distance_of(streamer_pos, subscriber_pos) > 0);
}

#[test]
fn path_exists_for_streamer_when_reachable() {
    let mut game = GameWorld::new();
    let tiled_tile_pos = TilePos::new(63, 49);

    let source_entity = game.find(EntityType::Streamer);

    let path = game.get_path_from(source_entity, tiled_tile_pos);
    assert_ne!(path.len(), 0);
}

#[test]
fn tile_pos_exists_in_ground_graph() {
    let mut game = GameWorld::new();
    let tiled_tile_pos = TilePos::new(63, 49);

    assert!(game.tile_has_neighbors(GraphType::Ground, tiled_tile_pos));
}

#[test]
fn heighted_tile_transform_matches_tile_transform() {
    let mut game = GameWorld::new();

    let tile_pos = TilePos::new(69, 20);
    let tile_height: usize = 19;

    let heighted_tile_pos = HeightedTilePos::new(tile_pos.clone(), tile_height as u32);

    let expected_transform = to_bevy_transform(&tile_pos, game.map_info(tile_height));
    let actual_transform = heighted_tile_pos.transform(game.map_info(tile_height));

    assert_eq!(expected_transform, actual_transform);
}

//  		(Key = 1.2.1.1.)
#[test]
fn streamer_arrives_to_fruit_at_higher_height() {
    let mut world = GameWorld::new();
    let source_entity = world.find(EntityType::Streamer);
    let target_entity = world.find(EntityType::Fruit);
    assert!(world.height_of(target_entity) > world.height_of(source_entity));
    world.travel_to(source_entity, target_entity);
    world.has_reached_tile(source_entity, target_entity);
}

//  		(Key = 1.2.2.1.)
#[test]
fn streamer_arrives_to_fruit_at_lower_height() {
    let mut world = GameWorld::new();
    let source_entity = world.find(EntityType::Streamer);
    let target_entity = world.find(EntityType::Fruit);
    assert!(world.height_of(target_entity) < world.height_of(source_entity));
    world.travel_to(source_entity, target_entity);
    world.has_reached_tile(source_entity, target_entity);
}

//  		(Key = 1.2.3.1.)
#[test]
fn streamer_arrives_to_fruit_at_same_height() {
    let mut world = GameWorld::new();
    let source_entity = world.find(EntityType::Streamer);
    let target_entity = world.find(EntityType::Fruit);
    assert!(world.height_of(target_entity) == world.height_of(source_entity));
    world.travel_to(source_entity, target_entity);
    world.has_reached_tile(source_entity, target_entity);
}

//  		(Key = 1.3.1.1.)
#[test]
fn streamer_arrives_to_crop_at_higher_height() {
    let mut world = GameWorld::new();
    let source_entity = world.find(EntityType::Streamer);
    let target_entity = world.find(EntityType::Crop);
    assert!(world.height_of(target_entity) > world.height_of(source_entity));
    world.travel_to(source_entity, target_entity);
    world.has_reached_tile(source_entity, target_entity);
}

//  		(Key = 1.3.2.1.)
#[test]
fn streamer_arrives_to_crop_at_lower_height() {
    let mut world = GameWorld::new();
    let source_entity = world.find(EntityType::Streamer);
    let target_entity = world.find(EntityType::Crop);
    assert!(world.height_of(target_entity) < world.height_of(source_entity));
    world.travel_to(source_entity, target_entity);
    world.has_reached_tile(source_entity, target_entity);
}

//  		(Key = 1.3.3.1.)
#[test]
fn streamer_arrives_to_crop_at_same_height() {
    let mut world = GameWorld::new();
    let source_entity = world.find(EntityType::Streamer);
    let target_entity = world.find(EntityType::Crop);
    assert!(world.height_of(target_entity) == world.height_of(source_entity));
    world.travel_to(source_entity, target_entity);
    world.has_reached_tile(source_entity, target_entity);
}

//  		(Key = 1.4.1.1.)
#[test]
fn streamer_arrives_at_higher_tile() {
    let mut world = GameWorld::new();
    let source_entity = world.find(EntityType::Streamer);
    let target_entity = world.tile_at_position(TilePos::new(44, 64), STREAMER_LAYER_NUM as u32 + 1);
    assert!(world.height_of(target_entity) > world.height_of(source_entity));
    world.travel_to(source_entity, target_entity);
    world.has_reached_tile(source_entity, target_entity);
    world.has_reached_tile_transform(
        source_entity,
        target_entity,
        STREAMER_LAYER_NUM,
        STREAMER_LAYER_NUM + 1,
    );
}

#[test]
fn streamer_wont_move_if_at_target() {
    let mut world = GameWorld::new();
    let source_entity = world.find(EntityType::Streamer);
    let source_tilepos = world.get_tile_pos_from(source_entity);
    let target_entity = world.tile_at_position(source_tilepos, STREAMER_LAYER_NUM as u32);
    let target_tilepos = world.get_tile_pos_from(target_entity);

    assert_eq!(source_tilepos, target_tilepos);
    assert_eq!(
        world.height_of(target_entity),
        world.height_of(source_entity)
    );

    world.travel_to(source_entity, target_entity);
    assert_eq!(world.get_tile_pos_from(source_entity), source_tilepos);
    world.has_reached_tile(source_entity, target_entity);
    world.has_reached_tile_transform(
        source_entity,
        target_entity,
        STREAMER_LAYER_NUM,
        STREAMER_LAYER_NUM,
    );
}

////  		(Key = 1.4.1.2.)
//#[test]
//fn test_case_8() {
//   let mut world = GameWorld::new();
//    let source_entity = world.spawn(EntityType::Streamer);
//   Destination Entity      :  Tile
//   assert!(world.height_of(target_entity) > world.height_of(source_entity));
//   world.travel_to(source_entity, target_entity);
//    assert!(!world.has_reached(source_entity, target_entity));
//}
//
////  		(Key = 1.4.2.1.)
//#[test]
//fn test_case_9() {
//   let mut world = GameWorld::new();
//    let source_entity = world.spawn(EntityType::Streamer);
//   Destination Entity      :  Tile
//   assert!(world.height_of(target_entity) < world.height_of(source_entity));
//   world.travel_to(source_entity, target_entity);
//    assert!(world.has_reached(source_entity, target_entity));
//}
//
//// 		(Key = 1.4.2.2.)
//#[test]
//fn test_case_10() {
//   let mut world = GameWorld::new();
//    let source_entity = world.spawn(EntityType::Streamer);
//   Destination Entity      :  Tile
//   assert!(world.height_of(target_entity) < world.height_of(source_entity));
//   world.travel_to(source_entity, target_entity);
//    assert!(!world.has_reached(source_entity, target_entity));
//}
//
//// 		(Key = 1.4.3.1.)
//#[test]
//fn test_case_11() {
//   let mut world = GameWorld::new();
//    let source_entity = world.spawn(EntityType::Streamer);
//   Destination Entity      :  Tile
//   assert!(world.height_of(target_entity) == world.height_of(source_entity));
//   world.travel_to(source_entity, target_entity);
//    assert!(world.has_reached(source_entity, target_entity));
//}
//
//// 		(Key = 1.4.3.2.)
//#[test]
//fn test_case_12() {
//   let mut world = GameWorld::new();
//    let source_entity = world.spawn(EntityType::Streamer);
//   Destination Entity      :  Tile
//   assert!(world.height_of(target_entity) == world.height_of(source_entity));
//   world.travel_to(source_entity, target_entity);
//    assert!(!world.has_reached(source_entity, target_entity));
//}

// 		(Key = 2.1.2.1.)
#[test]
fn chatter_arrives_to_streamer() {
    let mut world = GameWorld::new();
    let source_entity = world.find(EntityType::Chatter);
    let target_entity = world.find(EntityType::Streamer);
    assert!(world.height_of(target_entity) < world.height_of(source_entity));
    world.travel_to(source_entity, target_entity);
    world.has_reached_tile(source_entity, target_entity);
}

// 		(Key = 3.1.1.2.)
#[test]
fn subscriber_arrives_to_streamer() {
    let mut world = GameWorld::new();
    let source_entity = world.find(EntityType::Subscriber);
    let target_entity = world.find(EntityType::Streamer);
    assert!(world.height_of(target_entity) > world.height_of(source_entity));
    world.travel_to(source_entity, target_entity);
    world.has_reached_tile(source_entity, target_entity);
}

#[test]
fn node_data_from_ground_tiles_works() {
    let mut world = GameWorld::new();

    let heighted_tiles = world
        .app
        .world
        .query::<(&TilePos, &LayerNumber)>()
        .iter(&world.app.world)
        .map(|tile_entry| HeightedTilePos::new(*tile_entry.0, tile_entry.1 .0 as u32))
        .collect::<Vec<HeightedTilePos>>();

    let map_layer_information = world
        .app
        .world
        .query::<(&TilemapGridSize, &TilemapType, &Transform)>()
        .iter(&world.app.world)
        .map(|layer_info| (*layer_info.0, *layer_info.1, *layer_info.2))
        .collect::<Vec<(TilemapGridSize, TilemapType, Transform)>>();

    let number_of_tiles = unique_tiles_from(&heighted_tiles).len();
    let node_data = NodeData::from_ground_tiles(&heighted_tiles, map_layer_information);

    assert_eq!(number_of_tiles, node_data.0.len());
}
