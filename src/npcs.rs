use std::collections::HashMap;

use bevy::{ecs::system::Query, prelude::*};

use crate::{map::{Factions, MapIcon}, ships::look_for_enemies};

#[derive(Component, Debug)]
pub struct NonPlayableShip;

#[derive(Resource, Debug)]
pub struct ShipAiCooldownTimer(pub HashMap<Entity, Timer>);

// npses is a plural to NPS (NonPlayableShip). (:
pub fn ships_ai(
    mut commands: Commands,
    npses: Query<(Entity, &MapIcon)>,
    mut cooldown_timer: If<ResMut<ShipAiCooldownTimer>>,
    time: Res<Time>,
) {
    for (entity, icon) in npses {
        if icon.get_faction() == Factions::Player {
            continue;
        }
        if let Some(timer) = (*cooldown_timer).0.get_mut(&entity) {
            timer.tick(time.delta());
            if timer.is_finished() {
                let pos = icon.get_pos();

                look_for_enemies(&mut commands, (entity, icon), &npses, pos);
                
                timer.reset();
            }
        } else {
            (*cooldown_timer)
                .0
                .insert(entity, Timer::from_seconds(3., TimerMode::Once));
        }
    }
}
