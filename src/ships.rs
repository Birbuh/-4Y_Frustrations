use bevy::{ecs::{component::Component, system::entity_command::remove}, math::DVec2, prelude::*};

use crate::{
    map::{
        EntityPosInASector, Factions, IconType, MapIcon, MapVisible, Order, RelationType, Velocity,
        is_close_enough,
    },
    npcs::NonPlayableShip,
};

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
        max_vel: f64,
        acceleration: f64,
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
    pub repair_speed: i32,
}

impl FighterHealth {
    pub fn new(max_health: i32, shields: i32, repair_speed: i32) -> Self {
        Self {
            engines: max_health,
            core: max_health,
            weapons: max_health,
            repair_speed,
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

    pub fn estimate_wreckage(&self) -> Wreckage {
        let mut wreckage = ["", ""];
        if self.are_engines_wrecked() {
            wreckage[0] = "engines";
        }
        if self.are_weapons_wrecked() {
            wreckage[1] = "weapons";
        }
        match wreckage {
            ["engines", "weapons"] => Wreckage::EnginesWeapons,
            ["engines", ""] => Wreckage::Engines,
            ["", "weapons"] => Wreckage::Weapons,
            _ => Wreckage::None,
        }
    }
}

pub fn spawn_ship(
    commands: &mut Commands,
    sector_id: u16,
    pos: DVec2,
    owner: Factions,
    max_vel: f64,
    acceleration: f64,
    shields: i32,
    repair_speed: i32,
    max_health: i32,
    damage: i32,
    relation_type: RelationType,
    image: Handle<Image>,
) {
    let icon = IconType::Ship(ShipType::Fighter(Fighter::new(
        sector_id,
        pos,
        owner,
        max_vel,
        acceleration,
        shields,
        repair_speed,
        max_health,
        damage,
    )));
    commands
        .spawn((
            Sprite::from_image(image),
            icon.clone(),
            owner,
            EntityPosInASector {
                sector: sector_id,
                pos,
            },
            MapVisible,
            Transform::from_translation(pos.clone().as_vec2().extend(0.)),
            Velocity::ZERO,
            relation_type.clone(),
        ))
        .with_children(|parent| {
            parent.spawn((
                MapIcon::new(icon.clone(), relation_type.clone()),
                // MapRoute::new(pos.clone(), DVec2::new(2070., -2800.)),
                // Order::Fly,
            ));
        });
}

#[derive(Component, Clone, Debug, PartialEq)]
pub struct Fighter {
    pub sector_id: Option<u16>,
    pub pos: DVec2, // IMPORTANT: This is the position INSIDE THE SECTOR, not the whole map.
    pub owner: Factions,
    pub max_vel: f64,
    pub acceleration: f64,
    // pub cargo: Vec<(ProductType, Capacity)>,
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
        max_vel: f64,
        acceleration: f64,
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
            // cargo: Vec::new(),
            shields,
            health: FighterHealth::new(max_health, shields, repair_speed),
            max_health,
            damage,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ShipType {
    Fighter(Fighter),
}

pub fn look_for_enemies(
    commands: &mut Commands,
    ship: (Entity, &MapIcon),
    other_ships: &Query<(Entity, &MapIcon)>,
    ship_pos: DVec2,
) {
    let (entity, icon) = ship;
    let ship_owner = icon.get_faction();
    match ship_owner {
        Factions::TestAlly => {
            for (entity2, other_icon) in other_ships {
                let other_ship_pos = other_icon.get_pos();
                if is_close_enough(ship_pos, other_ship_pos, 100.) {
                    if entity == entity2 {
                        continue;
                    }
                    match other_icon.get_faction() {
                        Factions::TestEnemy => {
                            commands.entity(entity).insert(Order::Attack(entity2));
                        }
                        _ => {}
                    }
                }
            }
        }
        Factions::TestEnemy => {
            for (entity2, other_icon) in other_ships {
                let other_ship_pos = other_icon.get_pos();
                if is_close_enough(ship_pos, other_ship_pos, 100.) {
                    if entity == entity2 {
                        continue;
                    }
                    match other_icon.get_faction() {
                        Factions::Player | Factions::TestAlly => {
                            commands.entity(entity).insert(Order::Attack(entity2));
                        }
                        _ => {}
                    }
                }
            }
        }
        Factions::Player => {} // there will be another function for this
    }
}

#[derive(PartialEq, Eq)]
pub enum Wreckage {
    Engines,
    Weapons,
    EnginesWeapons,
    Boom,
    None,
}

pub fn update_fighter_texture(
    commands: &mut Commands,
    assets: &Res<AssetServer>,
    entity_tuple: (Entity, Factions, Wreckage),
) {
    let (entity, owner, wreckage) = entity_tuple;
    let image: Handle<Image> = match owner {
        Factions::TestAlly => match wreckage {
            Wreckage::Engines => assets.load("icons/fighter_ally-noengines.png"),
            Wreckage::Weapons => assets.load("icons/fighter_ally-noweapons.png"),
            Wreckage::EnginesWeapons => assets.load("icons/fighter_ally-noengines.png"),
            Wreckage::Boom => assets.load("icons/boom.png"),
            Wreckage::None => return,
        },
        Factions::TestEnemy => match wreckage {
            Wreckage::Engines => assets.load("icons/fighter_enemy-noengines.png"),
            Wreckage::Weapons => assets.load("icons/fighter_enemy-noweapons.png"),
            Wreckage::EnginesWeapons => assets.load("icons/fighter_enemy-noengines.png"),
            Wreckage::Boom => assets.load("icons/boom.png"),
            Wreckage::None => return,
        },
        Factions::Player => return,
    };
    commands
        .entity(entity)
        .queue_silenced(remove::<Sprite>())
        .insert(Sprite::from_image(image));
}
