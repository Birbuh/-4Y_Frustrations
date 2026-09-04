use bevy::{ecs::component::Component, math::DVec2};

use crate::{map::Factions, products::ProductType};

pub struct Capacity {
    max_capacity: f32,
    used_capacity: f32,
}

impl Capacity {
    pub fn new(capacity: f32) -> Self {
        Self { max_capacity: capacity, used_capacity: 0. }
    }
}


#[derive(Component)]
pub struct Fighter {
    pub sector_id: u16,
    pub pos: DVec2, // IMPORTANT: This is the position INSIDE THE SECTOR, not the whole map.
    pub owner: Factions,
    pub cargo: Vec<(ProductType, Capacity)>,
}

impl Fighter {
    pub fn new(sector_id: u16, pos: DVec2, owner: Factions) -> Self {
        Self {
            sector_id,
            pos,
            owner,
            cargo: Vec::new(),
        }
    }
}

pub enum ShipType {
    Fighter(Fighter),
}