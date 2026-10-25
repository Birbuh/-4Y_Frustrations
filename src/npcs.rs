//! Periodic decisions for non-player ships.
//!
//! AI timers are keyed by the child entities carrying [`MapIcon`], which also
//! receive [`Order`] components. Tick timers before decisions and reset them
//! after both AI and order execution have inspected their completion state.

use std::collections::HashMap;

use bevy::{ecs::system::Query, prelude::*};

use crate::{
    map::{Factions, MapIcon, MapRoute, Order},
    ships::look_for_enemies,
};

/// Per-ship decision timers keyed by the entity carrying its `MapIcon`.
///
/// Timers are created lazily by [`ships_ai`]. Despawning a ship does not
/// currently remove its entry from this resource.
#[derive(Resource, Debug)]
pub struct ShipAiCooldownTimer(pub HashMap<Entity, Timer>);

/// Marks a ship that has already been assigned its one-time flee order.
///
/// This marker remains after the resulting flight and suppresses further
/// automatic target searches.
#[derive(Component)]
pub struct Fleed;

// `npses` contains the logical child entities for all ships, including the player.
/// Assigns flee or attack orders when a non-player ship's timer finishes.
///
/// A ship with destroyed weapons flees once. Other ships search for enemies
/// when they have no route and have not fled. Existing orders alone do not
/// prevent a new search.
pub fn ships_ai(
    mut commands: Commands,
    npses: Query<(
        Entity,
        &MapIcon,
        Option<&Fleed>,
        Option<&MapRoute>,
        Option<&Order>,
    )>,
    mut cooldown_timer: If<ResMut<ShipAiCooldownTimer>>,
) {
    for (entity, icon, fleed, route, _order) in npses {
        if icon.get_faction() == Factions::Player {
            continue;
        }
        if let Some(timer) = (*cooldown_timer).0.get_mut(&entity) {
            if timer.is_finished() {
                let pos = icon.get_pos();
                if icon.get_health().are_weapons_wrecked() && fleed.is_none() {
                    commands.entity(entity).insert(Order::Flee).insert(Fleed);
                } else if fleed.is_none() && route.is_none() {
                    look_for_enemies(&mut commands, (entity, icon), &npses, pos);
                }
            }
        } else {
            // New ships wait one decision interval before their first AI action.
            (*cooldown_timer)
                .0
                .insert(entity, Timer::from_seconds(0.5, TimerMode::Once));
        }
    }
}

/// Advances all decision timers using the current simulation time step.
pub fn tick_ai_cooldown_timer(mut timers: ResMut<ShipAiCooldownTimer>, time: Res<Time>) {
    for timer in timers.0.values_mut() {
        timer.tick(time.delta());
    }
}

/// Restarts finished decision timers after their consumers have run.
pub fn reset_finished_ai_cooldown_timer(mut timers: ResMut<ShipAiCooldownTimer>) {
    for timer in timers.0.values_mut() {
        if timer.is_finished() {
            timer.reset();
        }
    }
}
