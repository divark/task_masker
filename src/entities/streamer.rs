use bevy::prelude::*;
use bevy_ecs_tilemap::{
    prelude::{IsoCoordSystem, TilemapGridSize, TilemapSize, TilemapType},
    tiles::TilePos,
};

use crate::map::tiled::tiledpos_to_tilepos;

use super::MovementType;

#[derive(Component)]
pub struct StreamerLabel;

#[derive(Bundle)]
pub struct Streamer {
    #[bundle]
    label: StreamerLabel,
    sprites: SpriteSheetBundle,
    movement_type: MovementType,
}

#[derive(Component)]
pub struct AnimationIndices {
    first: usize,
    last: usize,
}

pub fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(
        &AnimationIndices,
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
    )>,
) {
    for (indices, mut timer, mut sprite) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            sprite.index = if sprite.index == indices.last {
                indices.first
            } else {
                sprite.index + 1
            };
        }
    }
}

#[derive(Component, Deref, DerefMut)]
pub struct AnimationTimer(Timer);

pub fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    map_information: Query<
        (&Transform, &TilemapType, &TilemapGridSize, &TilemapSize),
        (Added<TilemapType>, Added<TilemapGridSize>),
    >,
    streamer_query: Query<(), With<StreamerLabel>>,
) {
    if !streamer_query.is_empty() {
        return;
    }

    let texture_handle = asset_server.load("entities/caveman-walk-down-left.png");
    let texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::new(15.0, 16.0), 4, 1, None, None);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    // Use only the subset of sprites in the sheet that make up the run animation
    let animation_indices = AnimationIndices { first: 0, last: 3 };

    if map_information.is_empty() {
        return;
    }

    let (map_transform, map_type, grid_size, map_size) = map_information
        .iter()
        .nth(1)
        .expect("Could not load map information. Is world loaded?");

    let streamer_location = TilePos { x: 1, y: 2 };
    let streamer_location = streamer_location
        .center_in_world(grid_size, map_type)
        .extend(25.0);
    let streamer_transform = *map_transform * Transform::from_translation(streamer_location);

    commands.spawn((
        Streamer {
            label: StreamerLabel,
            sprites: SpriteSheetBundle {
                texture_atlas: texture_atlas_handle,
                sprite: TextureAtlasSprite::new(animation_indices.first),
                transform: streamer_transform,
                ..default()
            },
            movement_type: MovementType::Walk,
        },
        animation_indices,
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
    ));
}
