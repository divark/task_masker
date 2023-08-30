use bevy::prelude::*;

use crate::{
    entities::MovementType,
    map::path_finding::{Direction, Path},
};

#[derive(Component, Deref, DerefMut)]
pub struct AnimationTimer(Timer);

#[derive(Component)]
pub struct AnimationIndices {
    row_num: usize,
    num_cols: usize,
}

pub fn insert_animation_information(
    moving_entities: Query<(Entity, &MovementType), Added<MovementType>>,
    mut commands: Commands,
) {
    for (moving_entity, entity_type) in &moving_entities {
        let (row_num, num_cols) = match entity_type {
            MovementType::Walk => (5, 4),
            MovementType::Fly => (0, 8),
            MovementType::Swim => (0, 1),
        };

        commands.entity(moving_entity).insert((
            AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
            AnimationIndices { row_num, num_cols },
            TextureAtlasSprite::new(row_num * num_cols),
        ));
    }
}

fn ground_directional_index_from(direction: &Direction) -> usize {
    match direction {
        Direction::BottomLeft => 7,
        Direction::BottomRight => 8,
        Direction::TopLeft => 5,
        Direction::TopRight => 6,
    }
}

fn fly_directional_index_from(direction: &Direction) -> usize {
    match direction {
        Direction::BottomLeft | Direction::TopLeft => 0,
        Direction::BottomRight | Direction::TopRight => 1,
    }
}

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
        animation_indices.row_num = direction_to_row_index(entity_direction, entity_type);
        entity_spritesheet.index = animation_indices.row_num;
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
fn bound_add_for_index(index: usize, row_num: usize, num_cols: usize) -> usize {
    let lower_bound = ((row_num + 1) * num_cols) - 1;
    let upper_bound = lower_bound + num_cols;

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
            entity_spritesheet.index = animation_indices.row_num;
            continue;
        }

        timer.tick(time.delta());
        if !timer.just_finished() {
            continue;
        }

        entity_spritesheet.index = bound_add_for_index(
            entity_spritesheet.index,
            animation_indices.row_num,
            animation_indices.num_cols,
        );
    }
}
