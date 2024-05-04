use bevy::prelude::*;

use bevy_ecs_tilemap::prelude::*;
use task_masker::entities::{chatter::*, streamer::*, subscriber::*};
use task_masker::map::plugins::PathFindingPlugin;
use task_masker::map::tiled::spawn_tiles_from_tiledmap;
use task_masker::ui::plugins::ChattingPlugin;
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
                speak_to_streamer_from_subscriber,
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
                speak_to_streamer_from_chatter,
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
        app.add_plugins(MinimalPlugins);
        app.add_plugins(MockTiledMapPlugin);
        // TODO: MovementTimer should update every 0 seconds for instant results.
        app.add_plugins(PathFindingPlugin);
        app.add_plugins(ChattingPlugin);

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
        todo!();
    }

    /// Returns a boolean representing whether two Entities
    /// co-exist in the same location.
    pub fn has_reached(&mut self, source_entity: Entity, target_entity: Entity) -> bool {
        todo!()
    }
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
