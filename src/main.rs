use bevy::{prelude::*, window::WindowResolution};
use bevy_ecs_tilemap::prelude::*;
use entities::streamer::{animate_sprite, spawn_player};
use ui::{plugins::StartupScreenPlugin};

mod entities;
mod map;
mod ui;

#[derive(Default, Debug, Hash, PartialEq, Eq, Clone, Copy, States)]
pub enum GameState {
    #[default]
    Start,
    InGame,
    End
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
                    watch_for_changes: true,
                    ..default()
                }),
        )
        .add_state::<GameState>()
        .add_plugin(TilemapPlugin)
        .add_plugin(map::plugins::TiledMapPlugin)
        .add_plugin(map::plugins::PathFindingPlugin)
        .add_plugin(StartupScreenPlugin)
        .add_startup_system(spawn_camera)
        .add_system(spawn_map.in_schedule(OnEnter(GameState::InGame)))
        .add_systems((spawn_player, map::camera::movement, animate_sprite).in_set(OnUpdate(GameState::InGame)))
        //.add_system(spawn_player)
        //.add_system(map::camera::movement)
        //.add_system(animate_sprite)
        .run();
}
