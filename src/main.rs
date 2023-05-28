use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use entities::streamer::{spawn_player, animate_sprite};
use map::path_finding::create_fly_graph;

mod map;
mod entities;

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
        // TODO: Incorporate a Loading state to make this line behave as intended:
        // https://github.com/NiklasEi/bevy_asset_loader
        .add_startup_systems((startup, spawn_player).before(create_fly_graph))
        .add_system(map::camera::movement)
        .add_system(animate_sprite)
        .run();
}
