use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use entities::streamer::{animate_sprite, spawn_player};
use map::path_finding::{create_ground_graph, insert_pathing_information, update_movement_target, move_entities, move_streamer, move_streamer_on_spacebar};

mod entities;
mod map;

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let map_handle: Handle<map::tiled::TiledMap> = asset_server.load("map2.tmx");

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
        .add_event::<TilePos>()
        .add_startup_system(startup)
        .add_systems((
            spawn_player,
            create_ground_graph,
            insert_pathing_information,
            update_movement_target,
            move_entities,
            move_streamer,
            move_streamer_on_spacebar,
        ))
        //.add_system(create_ground_graph)
        .add_system(map::camera::movement)
        .add_system(animate_sprite)
        .run();
}
