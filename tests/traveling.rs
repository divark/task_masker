use bevy::prelude::*;

use bevy_ecs_tilemap::prelude::*;
use task_masker::entities::{chatter::*, streamer::*, subscriber::*, MovementType};
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
    pub fn spawn(&mut self, entity_type: MovementType) -> Entity {
        match entity_type {
            MovementType::Walk => {
                self.app.add_plugins(MockStreamerPlugin);

                todo!()
            }
            MovementType::Swim => {
                self.app.add_plugins(MockSubscriberPlugin);

                todo!()
            }
            MovementType::Fly => {
                self.app.add_plugins(MockChatterPlugin);

                todo!()
            }
        }
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
