use bevy::prelude::*;

use bevy_ecs_tilemap::prelude::*;
use task_masker::entities::{chatter::*, streamer::*, subscriber::*};
use task_masker::map::path_finding::{tilepos_to_idx, GraphType, NodeEdges, Path};
use task_masker::map::plugins::{PathFindingPlugin, TilePosEvent};
use task_masker::map::tiled::spawn_tiles_from_tiledmap;
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
                mock_spawn_player,
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
                mock_replace_chatter,
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

enum EntityType {
    Streamer,
    Chatter,
    Subscriber,
    Fruit,
    Crop,
    //Tile,
}

/// A convenience abstraction of Task Masker
/// represented as what is needed to create
/// one of its worlds.
struct GameWorld {
    pub app: App,
}

impl GameWorld {
    pub fn new() -> Self {
        let mut app = App::new();

        app.init_state::<GameState>();
        app.insert_state(GameState::InGame);
        app.add_plugins(MinimalPlugins);
        app.add_plugins(MockTiledMapPlugin);
        // TODO: MovementTimer should update every 0 seconds for instant results.
        app.add_plugins(PathFindingPlugin);

        app.update();

        Self { app }
    }

    /// Returns a reference to the Entity just created
    /// based on its type.
    pub fn spawn(&mut self, entity_type: EntityType) -> Entity {
        match entity_type {
            EntityType::Streamer => {
                self.app.add_plugins(MockStreamerPlugin);

                self.app.update();

                return self
                    .app
                    .world
                    .query::<(Entity, &StreamerLabel)>()
                    .get_single(&self.app.world)
                    .expect("spawn: Streamer was not found after trying to spawn it.")
                    .0;
            }
            EntityType::Subscriber => {
                self.app.add_plugins(MockSubscriberPlugin);

                self.app.update();

                return self
                    .app
                    .world
                    .query::<(Entity, &SubscriberLabel)>()
                    .get_single(&self.app.world)
                    .expect("spawn: Subscriber was not found after trying to spawn it.")
                    .0;
            }
            EntityType::Chatter => {
                self.app.add_plugins(MockChatterPlugin);

                self.app.update();

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

    /// Returns the Height represented as the Z value
    /// for some given Entity.
    pub fn height_of(&mut self, entity: Entity) -> f32 {
        self.app
            .world
            .query::<&Transform>()
            .get(&self.app.world, entity)
            .unwrap()
            .translation
            .z
    }

    /// Triggers the Source Entity to move to the location
    /// of the Target Entity.
    pub fn travel_to(&mut self, source_entity: Entity, target_entity: Entity) {
        let target_pos = self.get_tile_pos_from(target_entity);
        let source_path = match self.get_entity_type(source_entity) {
            EntityType::Streamer => {
                self.app.world.send_event(TilePosEvent::new(target_pos));

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

        for _i in 0..=source_path.len() {
            self.app.update();
        }
    }

    /// Returns a boolean representing whether two Entities
    /// co-exist in the same location.
    pub fn has_reached(&mut self, source_entity: Entity, target_entity: Entity) -> bool {
        let source_pos = self.get_tile_pos_from(source_entity);
        let target_pos = self.get_tile_pos_from(target_entity);
        match self.get_entity_type(source_entity) {
            EntityType::Subscriber => return self.next_to_land(source_pos),
            EntityType::Chatter => {
                return distance_of(source_pos, target_pos) == DIST_AWAY_FROM_STREAMER
            }
            _ => return distance_of(source_pos, target_pos) == 0,
        }
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

    /// Returns the TilePos for some given Entity.
    fn get_tile_pos_from(&mut self, entity: Entity) -> TilePos {
        self.app
            .world
            .query::<&TilePos>()
            .get(&self.app.world, entity)
            .unwrap()
            .clone()
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

        panic!("get_entity_type: Entity given unknown type.");
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

    let streamer = world.spawn(EntityType::Streamer);
    let chatter = world.spawn(EntityType::Chatter);

    assert!(world.height_of(chatter) > world.height_of(streamer));
}

#[test]
fn subscriber_lower_than_streamer() {
    let mut world = GameWorld::new();

    let streamer = world.spawn(EntityType::Streamer);
    let subscriber = world.spawn(EntityType::Subscriber);

    assert!(world.height_of(subscriber) < world.height_of(streamer));
}

#[test]
fn streamer_and_subscriber_far_away_by_default() {
    let mut world = GameWorld::new();

    let streamer = world.spawn(EntityType::Streamer);
    let subscriber = world.spawn(EntityType::Subscriber);

    let streamer_pos = world.get_tile_pos_from(streamer);
    let subscriber_pos = world.get_tile_pos_from(subscriber);

    assert_ne!(streamer_pos, subscriber_pos);
    assert!(distance_of(streamer_pos, subscriber_pos) > 0);
}

//  		(Key = 1.2.1.1.)
#[test]
fn test_case_1() {
    let mut world = GameWorld::new();
    let source_entity = world.spawn(EntityType::Streamer);
    let target_entity = world.spawn(EntityType::Fruit);
    assert!(world.height_of(target_entity) > world.height_of(source_entity));
    world.travel_to(source_entity, target_entity);
    assert!(world.has_reached(source_entity, target_entity));
}

//  		(Key = 1.2.2.1.)
#[test]
fn test_case_2() {
    let mut world = GameWorld::new();
    let source_entity = world.spawn(EntityType::Streamer);
    let target_entity = world.spawn(EntityType::Fruit);
    assert!(world.height_of(target_entity) < world.height_of(source_entity));
    world.travel_to(source_entity, target_entity);
    assert!(world.has_reached(source_entity, target_entity));
}

//  		(Key = 1.2.3.1.)
#[test]
fn test_case_3() {
    let mut world = GameWorld::new();
    let source_entity = world.spawn(EntityType::Streamer);
    let target_entity = world.spawn(EntityType::Fruit);
    assert!(world.height_of(target_entity) == world.height_of(source_entity));
    world.travel_to(source_entity, target_entity);
    assert!(world.has_reached(source_entity, target_entity));
}

//  		(Key = 1.3.1.1.)
#[test]
fn test_case_4() {
    let mut world = GameWorld::new();
    let source_entity = world.spawn(EntityType::Streamer);
    let target_entity = world.spawn(EntityType::Crop);
    assert!(world.height_of(target_entity) > world.height_of(source_entity));
    world.travel_to(source_entity, target_entity);
    assert!(world.has_reached(source_entity, target_entity));
}

//  		(Key = 1.3.2.1.)
#[test]
fn test_case_5() {
    let mut world = GameWorld::new();
    let source_entity = world.spawn(EntityType::Streamer);
    let target_entity = world.spawn(EntityType::Crop);
    assert!(world.height_of(target_entity) < world.height_of(source_entity));
    world.travel_to(source_entity, target_entity);
    assert!(world.has_reached(source_entity, target_entity));
}

//  		(Key = 1.3.3.1.)
#[test]
fn test_case_6() {
    let mut world = GameWorld::new();
    let source_entity = world.spawn(EntityType::Streamer);
    let target_entity = world.spawn(EntityType::Crop);
    assert!(world.height_of(target_entity) == world.height_of(source_entity));
    world.travel_to(source_entity, target_entity);
    assert!(world.has_reached(source_entity, target_entity));
}

////  		(Key = 1.4.1.1.)
//#[test]
//fn test_case_7() {
//   let mut world = GameWorld::new();
//    let source_entity = world.spawn(EntityType::Streamer);
//   Destination Entity      :  Tile
//   assert!(world.height_of(target_entity) > world.height_of(source_entity));
//   world.travel_to(source_entity, target_entity);
//    assert!(world.has_reached(source_entity, target_entity));
//}
//
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
fn test_case_13() {
    let mut world = GameWorld::new();
    let source_entity = world.spawn(EntityType::Chatter);
    let target_entity = world.spawn(EntityType::Streamer);
    assert!(world.height_of(target_entity) < world.height_of(source_entity));
    world.travel_to(source_entity, target_entity);
    assert!(world.has_reached(source_entity, target_entity));
}

// 		(Key = 2.1.3.1.)
#[test]
fn test_case_14() {
    let mut world = GameWorld::new();
    let source_entity = world.spawn(EntityType::Chatter);
    let target_entity = world.spawn(EntityType::Streamer);
    assert!(world.height_of(target_entity) == world.height_of(source_entity));
    world.travel_to(source_entity, target_entity);
    assert!(world.has_reached(source_entity, target_entity));
}

//// 		(Key = 2.4.2.1.)
//#[test]
//fn test_case_15() {
//   let mut world = GameWorld::new();
//    let source_entity = world.spawn(EntityType::Chatter);
//   Destination Entity      :  Tile
//   assert!(world.height_of(target_entity) < world.height_of(source_entity));
//   world.travel_to(source_entity, target_entity);
//    assert!(world.has_reached(source_entity, target_entity));
//}
//
//// 		(Key = 2.4.3.1.)
//#[test]
//fn test_case_16() {
//   let mut world = GameWorld::new();
//    let source_entity = world.spawn(EntityType::Chatter);
//   Destination Entity      :  Tile
//   assert!(world.height_of(target_entity) == world.height_of(source_entity));
//   world.travel_to(source_entity, target_entity);
//    assert!(world.has_reached(source_entity, target_entity));
//}

// 		(Key = 3.1.1.2.)
#[test]
fn test_case_17() {
    let mut world = GameWorld::new();
    let source_entity = world.spawn(EntityType::Subscriber);
    let target_entity = world.spawn(EntityType::Streamer);
    assert!(world.height_of(target_entity) > world.height_of(source_entity));
    world.travel_to(source_entity, target_entity);
    assert!(!world.has_reached(source_entity, target_entity));
}

// 		(Key = 3.1.3.2.)
#[test]
fn test_case_18() {
    let mut world = GameWorld::new();
    let source_entity = world.spawn(EntityType::Subscriber);
    let target_entity = world.spawn(EntityType::Streamer);
    assert!(world.height_of(target_entity) == world.height_of(source_entity));
    world.travel_to(source_entity, target_entity);
    assert!(!world.has_reached(source_entity, target_entity));
}

//// 		(Key = 3.4.1.2.)
//#[test]
//fn test_case_19() {
//   let mut world = GameWorld::new();
//    let source_entity = world.spawn(EntityType::Subscriber);
//   Destination Entity      :  Tile
//   assert!(world.height_of(target_entity) > world.height_of(source_entity));
//   world.travel_to(source_entity, target_entity);
//    assert!(!world.has_reached(source_entity, target_entity));
//}
//
//// 		(Key = 3.4.3.2.)
//#[test]
//fn test_case_20() {
//   let mut world = GameWorld::new();
//    let source_entity = world.spawn(EntityType::Subscriber);
//   Destination Entity      :  Tile
//   assert!(world.height_of(target_entity) == world.height_of(source_entity));
//   world.travel_to(source_entity, target_entity);
//    assert!(!world.has_reached(source_entity, target_entity));
//}
