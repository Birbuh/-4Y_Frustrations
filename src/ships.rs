use bevy::{ecs::component::Component, math::DVec2, prelude::*};

use crate::{map::Factions, products::ProductType};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Capacity {
    max_capacity: f32,
    used_capacity: f32,
}

impl Capacity {
    pub fn new(capacity: f32) -> Self {
        Self {
            max_capacity: capacity,
            used_capacity: 0.,
        }
    }
}

pub trait Ship {
    fn new(
        sector_id: u16,
        pos: DVec2,
        owner: Factions,
        max_vel: f32,
        acceleration: f32,
        shields: i32, 
        repair_speed: i32,
        max_health: i32,
        damage: i32,
    ) -> Self;
}

#[derive(Clone, Debug, PartialEq)]
pub struct FighterHealth {
    pub engines: i32,
    pub core: i32,
    pub weapons: i32,
    pub repair_speed: i32
}

impl FighterHealth {
    pub fn new(max_health: i32, repair_speed: i32) -> Self {
        Self {
            engines: max_health,
            core: max_health,
            weapons: max_health,
            repair_speed
        }
    }
    pub fn injure_engines(&mut self, amount: i32) {
        if self.engines - amount < 0 {
            self.engines = 0
        } else {
            self.engines -= amount
        }
    }
    
    pub fn injure_core(&mut self, amount: i32) {
        if self.core - amount < 0 {
            self.core = 0
        } else {
            self.core -= amount
        }
    } 

    pub fn injure_weapon_module(&mut self, amount: i32) {
        if self.weapons - amount < 0 {
            self.weapons = 0
        } else {
            self.weapons -= amount
        }
    }
    
    pub fn are_engines_wrecked(&self) -> bool {
        self.engines <= 0
    }

    pub fn are_weapons_wrecked(&self) -> bool {
        self.weapons <= 0
    }

    pub fn is_ship_wrecked(&self) -> bool {
        self.core <= 0
    }
}

#[derive(Component, Clone, Debug, PartialEq)]
pub struct Fighter {
    pub sector_id: Option<u16>,
    pub pos: DVec2, // IMPORTANT: This is the position INSIDE THE SECTOR, not the whole map.
    pub owner: Factions,
    pub max_vel: f32,
    pub acceleration: f32,
    pub cargo: Vec<(ProductType, Capacity)>,
    pub shields: i32,
    pub health: FighterHealth,
    pub max_health: i32,
    pub damage: i32,
}

impl Ship for Fighter {
    fn new(
        sector_id: u16,
        pos: DVec2,
        owner: Factions,
        max_vel: f32,
        acceleration: f32,
        shields: i32, 
        repair_speed: i32,
        max_health: i32,
        damage: i32,
    ) -> Self {
        Self {
            sector_id: Some(sector_id),
            pos,
            owner,
            max_vel,
            acceleration,
            cargo: Vec::new(),
            shields,
            health: FighterHealth::new(max_health, repair_speed),
            max_health,
            damage,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ShipType {
    Fighter(Fighter),
}
