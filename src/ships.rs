use bevy::{ecs::component::Component, math::DVec2, prelude::*};

use crate::{map::Factions, products::ProductType};

#[derive(Clone, Copy, Debug)]
pub struct Capacity {
    max_capacity: f32,
    used_capacity: f32,
}

impl Capacity {
    pub fn new(capacity: f32) -> Self {
        Self { max_capacity: capacity, used_capacity: 0. }
    }
}

pub trait Ship {
    fn new(sector_id: u16, pos: DVec2, owner: Factions) -> Self;

    // fn go_somewhere(&self, )
}

#[derive(Component, Clone, Debug)]
pub struct Fighter {
    pub sector_id: Option<u16>,
    pub pos: DVec2, // IMPORTANT: This is the position INSIDE THE SECTOR, not the whole map.
    pub owner: Factions,
    pub cargo: Vec<(ProductType, Capacity)>,
}

impl Ship for Fighter {
    fn new(sector_id: u16, pos: DVec2, owner: Factions) -> Self {
        Self {
            sector_id: Some(sector_id),
            pos,
            owner,
            cargo: Vec::new(),
        }
    }

    
}

#[derive(Clone, Debug)]
pub enum ShipType {
    Fighter(Fighter),
}