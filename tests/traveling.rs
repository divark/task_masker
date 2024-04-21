use bevy::prelude::*;

use bevy_ecs_tilemap::prelude::*;
use task_masker::map::plugins::PathFindingPlugin;
use task_masker::map::tiled::process_loaded_maps;
use task_masker::ui::plugins::ChattingPlugin;
use task_masker::GameState;
use task_masker::{
    entities::{
        plugins::{ChatterPlugin, StreamerPlugin, SubscriberPlugin},
        MovementType,
    },
    map::tiled::{spawn_map, TiledLoader, TiledMap},
};

#[derive(Default)]
pub struct NoRenderTiledMapPlugin;

impl Plugin for NoRenderTiledMapPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_asset::<TiledMap>()
            .register_asset_loader(TiledLoader)
            .add_systems(Startup, spawn_map)
            .add_systems(Update, process_loaded_maps);
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
        // TODO: Implement custom system that loads tmx map directly
        // without needing Asset Server, adding this into
        // NoRenderTiledMapPlugin.
        app.add_plugins(AssetPlugin::default());
        app.add_plugins(NoRenderTiledMapPlugin);
        app.add_plugins(PathFindingPlugin);
        app.add_plugins(ChattingPlugin);

        Self { app }
    }

    /// Returns a reference to the Entity just created
    /// based on its type.
    pub fn spawn(&mut self, entity_type: MovementType) -> Entity {
        match entity_type {
            MovementType::Walk => {
                self.app.add_plugins(StreamerPlugin);

                todo!()
            }
            MovementType::Swim => {
                self.app.add_plugins(SubscriberPlugin);

                todo!()
            }
            MovementType::Fly => {
                self.app.add_plugins(ChatterPlugin);

                todo!()
            }
        }
    }
}

#[test]
fn creating_gameworld_does_not_crash() {
    let mut world = GameWorld::new();

    assert_eq!(
        world
            .app
            .world
            .query::<&TilePos>()
            .iter(&world.app.world)
            .len(),
        1
    );
}
