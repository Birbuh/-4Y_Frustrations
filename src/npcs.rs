use std::collections::HashMap;

use bevy::{ecs::system::Query, prelude::*};

use crate::{
    map::{Factions, MapIcon},
    ships::look_for_enemies,
};

#[derive(Component, Debug)]
pub struct NonPlayableShip;

#[derive(Resource, Debug)]
pub struct ShipAiCooldownTimer(pub HashMap<Entity, Timer>);

// npses is a plural to NPS (NonPlayableShip). (:
pub fn ships_ai(
    mut commands: Commands,
    npses: Query<(Entity, &MapIcon)>,
    mut cooldown_timer: If<ResMut<ShipAiCooldownTimer>>,
) {
    for (entity, icon) in npses {
        if icon.get_faction() == Factions::Player {
            continue;
        }
        if let Some(timer) = (*cooldown_timer).0.get_mut(&entity) {
            if timer.is_finished() {
                let pos = icon.get_pos();
                look_for_enemies(&mut commands, (entity, icon), &npses, pos);
            }
        } else {
            (*cooldown_timer)
                .0
                .insert(entity, Timer::from_seconds(1., TimerMode::Once));
        }
    }
}

pub fn tick_ai_cooldown_timer(mut timers: ResMut<ShipAiCooldownTimer>, time: Res<Time>) {
    for timer in timers.0.values_mut() {
        timer.tick(time.delta());
    }
}

pub fn reset_finished_ai_cooldown_timer(mut timers: ResMut<ShipAiCooldownTimer>) {
    for timer in timers.0.values_mut() {
        if timer.is_finished() {
            timer.reset();
        }
    }
}
