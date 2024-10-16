use bevy::prelude::*;

use crate::entities::streamer::StreamerLabel;
use crate::entities::GameEntityType;
use crate::map::path_finding::Direction;
use crate::map::tilemap::*;

pub const CAMPFIRE_LAYER_NUM: usize = 20;

#[derive(Component)]
pub struct CampfireLabel;

/// Respawns Campfire without rendering components
pub fn replace_campfire_tile(
    tiles_query: Query<(Entity, &TileGridCoordinates, &Transform)>,
    mut commands: Commands,
) {
    for (campfire_entity, tile_pos, tile_transform) in &tiles_query {
        if tile_pos.z() != CAMPFIRE_LAYER_NUM {
            continue;
        }

        commands.entity(campfire_entity).despawn_recursive();
        commands.spawn((
            (CampfireLabel, *tile_transform, GameEntityType::Environment),
            tile_pos.clone(),
        ));
    }
}

/// Makes the Streamer face towards the Campfire when
/// next to it.
pub fn make_streamer_face_campfire(
    mut streamer_query: Query<
        (&TileGridCoordinates, &mut Direction),
        (With<StreamerLabel>, Changed<TileGridCoordinates>),
    >,
    campfire: Query<&TileGridCoordinates, With<CampfireLabel>>,
) {
    if streamer_query.is_empty() || campfire.is_empty() {
        return;
    }

    let (streamer_tilepos, mut streamer_direction) = streamer_query.single_mut();
    let campfire_tilepos = campfire.single();
    let left_of_campfire = TileGridCoordinates::new(campfire_tilepos.x(), campfire_tilepos.y() - 1);

    if *streamer_tilepos != left_of_campfire {
        return;
    }

    *streamer_direction = Direction::BottomRight;
}
