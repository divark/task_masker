use audio::plugins::BackgroundMusicPlugin;
use bevy::{prelude::*, window::WindowResolution};
use bevy_ecs_tilemap::prelude::*;
use entities::plugins::{ChatterPlugin, CropPlugin, FruitPlugin, StreamerPlugin};
use map::plugins::TiledCameraPlugin;
use ui::plugins::{ChattingPlugin, StartupScreenPlugin};
use visual::plugins::AnimationPlugin;

mod audio;
mod entities;
mod map;
mod ui;
mod visual;

#[derive(Default, Debug, Hash, PartialEq, Eq, Clone, Copy, States)]
pub enum GameState {
    #[default]
    Start,
    InGame,
    End,
}

fn spawn_map(mut commands: Commands, asset_server: Res<AssetServer>) {
    let map_handle: Handle<map::tiled::TiledMap> = asset_server.load("TM_v3.tmx");

    commands.spawn(map::tiled::TiledMapBundle {
        tiled_map: map_handle,
        render_settings: TilemapRenderSettings {
            render_chunk_size: UVec2::new(1280, 1),
            y_sort: true,
        },
        ..Default::default()
    });
}

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
        .add_state::<GameState>()
        .add_plugins(TilemapPlugin)
        .add_plugins(map::plugins::TiledMapPlugin)
        .add_plugins(map::plugins::PathFindingPlugin)
        .add_plugins(StartupScreenPlugin)
        .add_plugins(ChattingPlugin)
        .add_plugins(BackgroundMusicPlugin)
        .add_plugins(AnimationPlugin)
        .add_plugins(StreamerPlugin)
        .add_plugins(FruitPlugin)
        .add_plugins(CropPlugin)
        .add_plugins(ChatterPlugin)
        .add_plugins(TiledCameraPlugin)
        .run();
}
