use bevy::{prelude::*, window::WindowResolution};
use bevy_ecs_tilemap::prelude::*;
//use bevy_inspector_egui::quick::WorldInspectorPlugin;
use entities::streamer::{animate_sprite, spawn_player};
use ui::plugins::StartupScreenPlugin;

mod entities;
mod map;
mod ui;

#[derive(Default, Debug, Hash, PartialEq, Eq, Clone, Copy, States)]
pub enum GameState {
    #[default]
    Start,
    InGame,
    End,
}

fn spawn_map(mut commands: Commands, asset_server: Res<AssetServer>) {
    let map_handle: Handle<map::tiled::TiledMap> = asset_server.load("TM_v2.tmx");

    commands.spawn(map::tiled::TiledMapBundle {
        tiled_map: map_handle,
        ..Default::default()
    });
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn main() {
    App::new()
        .insert_resource(TilemapRenderSettings {
            render_chunk_size: UVec2::new(1280, 1),
            y_sort: true,
        })
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
        //.add_plugin(WorldInspectorPlugin::new())
        .add_systems(Startup, spawn_camera)
        .add_systems(Update, animate_sprite)
        .add_systems(
            Update,
            (spawn_player, map::camera::movement).run_if(in_state(GameState::InGame)),
        )
        .run();
}
