use bevy::prelude::*;
use task_masker::*;

use audio::plugins::BackgroundMusicPlugin;
use bevy::window::WindowResolution;
use bevy_ecs_tilemap::prelude::*;
use entities::plugins::{ChatterPlugin, CropPlugin, FruitPlugin, StreamerPlugin, SubscriberPlugin};
use map::plugins::{PathFindingPlugin, TiledCameraPlugin, TiledMapPlugin};
use ui::plugins::{ChattingPlugin, StartupScreenPlugin};
use visual::plugins::AnimationPlugin;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: String::from("Task Masker"),
                        resolution: WindowResolution::new(1280.0, 720.0),
                        ..Default::default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest())
                .set(AssetPlugin {
                    //watch_for_changes: true,
                    ..default()
                }),
        )
        .init_state::<GameState>()
        .add_plugins(TilemapPlugin)
        .add_plugins(TiledMapPlugin)
        .add_plugins(PathFindingPlugin)
        .add_plugins(StartupScreenPlugin)
        .add_plugins(ChattingPlugin)
        .add_plugins(BackgroundMusicPlugin)
        .add_plugins(AnimationPlugin)
        .add_plugins(StreamerPlugin)
        .add_plugins(FruitPlugin)
        .add_plugins(CropPlugin)
        .add_plugins(ChatterPlugin)
        .add_plugins(SubscriberPlugin)
        .add_plugins(TiledCameraPlugin)
        .run();
}
