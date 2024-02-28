use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::entities::streamer::StreamerLabel;
use crate::map::path_finding::*;
use crate::map::tiled::{tiled_to_tile_pos, to_bevy_transform, LayerNumber, TiledMapInformation};
use crate::ui::chatting::Msg;

use super::MovementType;

pub const CHATTER_LAYER_NUM: usize = 18;
pub const DIST_AWAY_FROM_STREAMER: usize = 2;

#[derive(Component)]
pub struct ChatterLabel;

#[derive(Component, PartialEq)]
pub enum ChatterStatus {
    Idle,
    Approaching,
    Speaking,
    Leaving,
}

#[derive(Component)]
pub struct WaitTimer(Timer);

#[derive(Component, Event, Clone)]
pub struct ChatMsg {
    pub name: String,
    pub msg: String,
}

#[derive(Bundle)]
pub struct ChatterBundle {
    label: ChatterLabel,
    sprite: SpriteSheetBundle,
    movement_type: MovementType,
    status: ChatterStatus,
}

pub fn replace_chatter(
    mut tiles_query: Query<(Entity, &LayerNumber, &TilePos, &TileTextureIndex)>,
    map_info_query: Query<
        (&Transform, &TilemapGridSize, &TilemapSize, &TilemapType),
        Added<TilemapGridSize>,
    >,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let map_information = map_info_query
        .iter()
        .find(|map_info| map_info.0.translation.z == CHATTER_LAYER_NUM as f32);

    if map_information.is_none() {
        return;
    }

    let (map_transform, grid_size, map_size, map_type) =
        map_information.expect("replace_chatter: Map information should exist by now.");

    let texture_handle = asset_server.load("BirdSprite (16x16).png");
    let chatter_texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::new(16.0, 16.0), 8, 3, None, None);
    let chatter_texture_atlas_handle = texture_atlases.add(chatter_texture_atlas);
    for (chatter_entity, layer_number, tile_pos, tile_texture_index) in &mut tiles_query {
        if layer_number.0 != CHATTER_LAYER_NUM {
            continue;
        }

        let map_info = TiledMapInformation::new(grid_size, map_size, map_type, map_transform);
        let tile_transform = to_bevy_transform(tile_pos, map_info);

        let chatter_sprite = SpriteSheetBundle {
            sprite: TextureAtlasSprite::new(tile_texture_index.0 as usize),
            texture_atlas: chatter_texture_atlas_handle.clone(),
            transform: tile_transform,
            ..default()
        };
        let chatter_tilepos = tiled_to_tile_pos(tile_pos.x, tile_pos.y, map_size);

        commands.entity(chatter_entity).despawn_recursive();
        commands.spawn((
            ChatterBundle {
                label: ChatterLabel,
                sprite: chatter_sprite,
                movement_type: MovementType::Fly,
                status: ChatterStatus::Idle,
            },
            chatter_tilepos,
        ));
    }
}

pub fn trigger_flying_to_streamer(
    mut chatter_msg: EventWriter<ChatMsg>,
    pressed_key: Res<Input<KeyCode>>,
) {
    if !pressed_key.pressed(KeyCode::G) {
        return;
    }

    let chat_msg = ChatMsg {
        name: String::from("Bob"),
        msg: String::from("Hello Caveman!"),
    };

    chatter_msg.send(chat_msg);
}

pub fn fly_to_streamer_to_speak(
    mut chatter_msg: EventReader<ChatMsg>,
    mut chatter: Query<
        (Entity, &TilePos, &mut Path, &mut ChatterStatus),
        (With<ChatterLabel>, Without<ChatMsg>),
    >,
    air_graph: Query<(&NodeEdges, &GraphType)>,
    streamer: Query<&TilePos, With<StreamerLabel>>,
    map_info: Query<&TilemapSize>,
    mut commands: Commands,
) {
    if air_graph.is_empty() || streamer.is_empty() || map_info.is_empty() {
        return;
    }

    let air_graph_edges = air_graph
        .iter()
        .find(|graph_elements| graph_elements.1 == &GraphType::Air)
        .expect("fly_to_streamer_to_speak: There should only be one air graph.");
    let streamer_tilepos = streamer
        .get_single()
        .expect("fly_to_streamer_to_speak: There should only be one streamer.");
    let map_size = map_info
        .iter()
        .next()
        .expect("fly_to_streamer_to_speak: There should be only one map.");
    for (chatter_entity, chatter_tilepos, mut chatter_path, mut chatter_status) in &mut chatter {
        if chatter_msg.is_empty() || *chatter_status != ChatterStatus::Idle {
            break;
        }

        let mut path_to_streamer = get_path(
            chatter_tilepos,
            streamer_tilepos,
            map_size,
            air_graph_edges.0,
        );

        // The chatter should not be directly on top of the
        // streamer, so we provide some distance by adjusting
        // the path to not go straight to the streamer.
        for _i in 0..DIST_AWAY_FROM_STREAMER {
            path_to_streamer.pop_back();
        }

        *chatter_path = path_to_streamer;

        commands
            .entity(chatter_entity)
            .insert(chatter_msg.read().next().unwrap().clone());

        *chatter_status = ChatterStatus::Approaching;
    }
}

pub fn speak_to_streamer(
    mut chatter_query: Query<
        (Entity, &ChatMsg, &Path, &mut ChatterStatus, &MovementType),
        Without<WaitTimer>,
    >,
    mut chat_msg_requester: EventWriter<Msg>,
    mut commands: Commands,
) {
    for (chatter_entity, chatter_msg, chatter_path, mut chatter_status, &chatter_type) in
        &mut chatter_query
    {
        if !chatter_path.0.is_empty() || *chatter_status != ChatterStatus::Approaching {
            continue;
        }

        commands
            .entity(chatter_entity)
            .insert(WaitTimer(Timer::from_seconds(60.0, TimerMode::Once)));

        *chatter_status = ChatterStatus::Speaking;
        chat_msg_requester.send(Msg {
            speaker_name: chatter_msg.name.clone(),
            speaker_role: chatter_type,
            msg: chatter_msg.msg.clone(),
        });
    }
}
