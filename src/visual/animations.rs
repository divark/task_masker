use bevy::prelude::*;

use crate::{
    entities::{streamer::StreamerLabel, MovementType},
    map::path_finding::{tilepos_to_idx, Direction, Path},
};

#[derive(Component, Deref, DerefMut)]
pub struct AnimationTimer(Timer);

#[derive(Component)]
pub struct AnimationIndices {
    start_idx: usize,
    end_idx: usize,
}

pub fn insert_animation_information(
    moving_entities: Query<(Entity, &MovementType), Added<MovementType>>,
    mut commands: Commands,
) {
    for (moving_entity, entity_type) in &moving_entities {
        let start_idx = match entity_type {
            MovementType::Walk => ground_directional_index_from(&Direction::BottomRight),
            MovementType::Fly => fly_directional_index_from(&Direction::BottomRight),
            MovementType::Swim => swim_directional_index_from(&Direction::BottomRight),
        };

        let end_idx = start_idx + movement_type_len(entity_type);

        commands.entity(moving_entity).insert((
            AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
            AnimationIndices { start_idx, end_idx },
            TextureAtlasSprite::new(start_idx),
        ));
    }
}

fn movement_type_len(entity_type: &MovementType) -> usize {
    match entity_type {
        MovementType::Walk => 4,
        MovementType::Fly => 8,
        MovementType::Swim => 1,
    }
}

fn ground_directional_index_from(direction: &Direction) -> usize {
    let num_ground_sprites_in_row = movement_type_len(&MovementType::Walk) as u32;

    match direction {
        Direction::BottomLeft => tilepos_to_idx(7, 0, num_ground_sprites_in_row),
        Direction::BottomRight => tilepos_to_idx(8, 0, num_ground_sprites_in_row),
        Direction::TopLeft => tilepos_to_idx(5, 0, num_ground_sprites_in_row),
        Direction::TopRight => tilepos_to_idx(6, 0, num_ground_sprites_in_row),
    }
}

fn fly_directional_index_from(direction: &Direction) -> usize {
    let num_flying_sprites_in_row = movement_type_len(&MovementType::Fly) as u32;

    match direction {
        Direction::BottomLeft | Direction::TopLeft => {
            tilepos_to_idx(1, 0, num_flying_sprites_in_row)
        }
        Direction::BottomRight | Direction::TopRight => {
            tilepos_to_idx(1, 0, num_flying_sprites_in_row)
        }
    }
}

fn swim_directional_index_from(_direction: &Direction) -> usize {
    let num_swim_sprites_in_row = movement_type_len(&MovementType::Swim) as u32;

    tilepos_to_idx(0, 0, num_swim_sprites_in_row)
}

fn direction_to_row_index(direction: &Direction, entity_type: &MovementType) -> usize {
    match entity_type {
        MovementType::Walk => ground_directional_index_from(direction),
        MovementType::Fly => fly_directional_index_from(direction),
        MovementType::Swim => swim_directional_index_from(direction),
    }
}

pub fn change_sprite_direction(
    mut moving_entities: Query<
        (
            &mut AnimationIndices,
            &mut TextureAtlasSprite,
            &MovementType,
            &Direction,
        ),
        Changed<Direction>,
    >,
) {
    for (mut animation_indices, mut entity_spritesheet, entity_type, entity_direction) in
        &mut moving_entities
    {
        animation_indices.start_idx = direction_to_row_index(entity_direction, entity_type);
        animation_indices.end_idx = animation_indices.start_idx + movement_type_len(entity_type);
        entity_spritesheet.index = animation_indices.start_idx;
    }
}

pub fn check_if_idle(
    mut moving_entities: Query<(&mut AnimationTimer, &Path), With<StreamerLabel>>,
) {
    for (mut timer, path) in &mut moving_entities {
        if path.is_empty() {
            timer.pause();
            timer.reset();
            continue;
        }

        timer.unpause();
    }
}

//TODO: Make Unit Tests to ensure indexes are calculated as intended.
fn bound_add_for_index(index: usize, lower_bound: usize, upper_bound: usize) -> usize {
    let new_index = index + 1;
    if new_index >= upper_bound {
        return lower_bound;
    }

    new_index
}

pub fn animate(
    mut moving_entities: Query<(
        &mut AnimationTimer,
        &mut AnimationIndices,
        &mut TextureAtlasSprite,
    )>,
    time: Res<Time>,
) {
    for (mut timer, animation_indices, mut entity_spritesheet) in &mut moving_entities {
        if timer.paused() {
            entity_spritesheet.index = animation_indices.start_idx;
            continue;
        }

        timer.tick(time.delta());
        if !timer.just_finished() {
            continue;
        }

        entity_spritesheet.index = bound_add_for_index(
            entity_spritesheet.index,
            animation_indices.start_idx,
            animation_indices.end_idx,
        );
    }
}
