use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use entities::streamer::{animate_sprite, spawn_player};
use map::path_finding::create_ground_graph;

mod entities;
mod map;

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let map_handle: Handle<map::tiled::TiledMap> = asset_server.load("map.tmx");

    commands.spawn(map::tiled::TiledMapBundle {
        tiled_map: map_handle,
        ..Default::default()
    });
}

fn main() {
    App::new()
        .insert_resource(TilemapRenderSettings {
            render_chunk_size: UVec2::new(20, 1),
        })
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: String::from("Tiled Map Editor Example"),
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
        .add_plugin(TilemapPlugin)
        .add_plugin(map::tiled::TiledMapPlugin)
        .add_startup_systems((startup, spawn_player))
        .add_system(create_ground_graph)
        .add_system(map::camera::movement)
        .add_system(animate_sprite)
        .run();
}
