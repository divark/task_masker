use bevy::prelude::*;

use crate::{
    entities::MovementType,
    map::path_finding::{tilepos_to_idx, Direction, Path, Target},
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
            MovementType::Fly => tilepos_to_idx(0, 0, 6),
            MovementType::Swim => tilepos_to_idx(0, 0, 1),
        };

        let end_idx = start_idx
            + match entity_type {
                MovementType::Walk => 4,
                MovementType::Fly => 6,
                MovementType::Swim => 1,
            };

        commands.entity(moving_entity).insert((
            AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
            AnimationIndices { start_idx, end_idx },
            TextureAtlasSprite::new(start_idx),
        ));
    }
}

fn ground_directional_index_from(direction: &Direction) -> usize {
    match direction {
        Direction::BottomLeft => tilepos_to_idx(7, 0, 4),
        Direction::BottomRight => tilepos_to_idx(8, 0, 4),
        Direction::TopLeft => tilepos_to_idx(5, 0, 4),
        Direction::TopRight => tilepos_to_idx(6, 0, 4),
    }
}

//TODO: Use tilepos_to_idx for this like ground_directional_index_from.
fn fly_directional_index_from(direction: &Direction) -> usize {
    match direction {
        Direction::BottomLeft | Direction::TopLeft => 0,
        Direction::BottomRight | Direction::TopRight => 1,
    }
}

//TODO: Use tilepos_to_idx for this like ground_directional_index_from.
fn swim_directional_index_from(direction: &Direction) -> usize {
    0
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
        //TODO: Make this +4 dependent upon entity type.
        animation_indices.end_idx = animation_indices.start_idx + 4;
        entity_spritesheet.index = animation_indices.start_idx;
    }
}

pub fn check_if_idle(mut moving_entities: Query<(&mut AnimationTimer, &Path)>) {
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
