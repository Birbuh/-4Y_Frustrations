use bevy::{
    ecs::{component::Component, system::entity_command::despawn},
    math::DVec2,
    prelude::*,
};
use bevy_rapier2d::{
    dynamics::{Ccd, GravityScale, RigidBody},
    geometry::{ActiveEvents, Collider, CollisionGroups, Group},
    pipeline::CollisionEvent,
    rapier::prelude::Vec2,
};

use crate::{map::{
    Factions, IconType, MapIcon, MapRoute, MapVisible, Order, RelationType, Velocity, is_close_enough, which_part_to_injure,
}, npcs::Fleed};

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
    pub fn new(max_health: i32, repair_speed: i32) -> Self {
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
            MapVisible,
            Transform::from_translation(pos.clone().as_vec2().extend(0.)),
            Velocity::ZERO,
            relation_type.clone(),
            RigidBody::Fixed,
            Collider::triangle(
                Vec2::new(64., -64.),
                Vec2::new(0., 64.),
                Vec2::new(-64., -64.),
            ),
            Ccd::enabled(),
            ActiveEvents::COLLISION_EVENTS,
            CollisionGroups::new(Group::GROUP_1, Group::GROUP_2),
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
    pub max_vel: f64, // max vel of the ship at this moment
    pub max_speed: f64, // max speed of the ship in overall
    pub acceleration: f64,
    // pub cargo: Vec<(ProductType, Capacity)>,
    pub shields: i32,
    pub max_shields: i32,
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
            max_speed: max_vel,
            acceleration,
            // cargo: Vec::new(),
            shields,
            max_shields: shields,
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

pub fn look_for_enemies(
    commands: &mut Commands,
    ship: (Entity, &MapIcon),
    other_ships: &Query<(Entity, &MapIcon, Option<&Fleed>, Option<&MapRoute>)>,
    ship_pos: DVec2,
) {
    let (entity, icon) = ship;
    let ship_owner = icon.get_faction();
    match ship_owner {
        Factions::TestAlly => {
            for (entity2, other_icon, _, _) in other_ships {
                if entity == entity2 {
                    continue;
                }
                let other_ship_pos = other_icon.get_pos();
                if is_close_enough(ship_pos, other_ship_pos, 2000.) {
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
            for (entity2, other_icon, _, _) in other_ships {
                if entity == entity2 {
                    continue;
                }
                let other_ship_pos = other_icon.get_pos();
                if is_close_enough(ship_pos, other_ship_pos, 2000.) {
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

#[derive(PartialEq, Eq, Debug)]
pub enum Wreckage {
    Engines,
    Weapons,
    EnginesWeapons,
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
            Wreckage::EnginesWeapons => assets.load("icons/fighter_ally-noengines-noweapons.png"),
            Wreckage::None => return,
        },
        Factions::TestEnemy => match wreckage {
            Wreckage::Engines => assets.load("icons/fighter_enemy-noengines.png"),
            Wreckage::Weapons => assets.load("icons/fighter_enemy-noweapons.png"),
            Wreckage::EnginesWeapons => assets.load("icons/fighter_enemy-noengines-noweapons.png"),
            Wreckage::None => return,
        },
        Factions::Player => return,
    };

    commands.queue(move |world: &mut World| {
        if let Ok(mut entity_mut) = world.get_entity_mut(entity) {
            entity_mut.insert(Sprite::from_image(image));
        }
    });
}

#[derive(Component)]
pub struct FighterHealthBarFill {
    pub ship: Entity,
    pub module: Option<usize>, // None -> shields; 0 -> core; 1 -> engines; 2 -> weapons
}

impl FighterHealthBarFill {
    const BAR_WIDTH: f32 = 10.;
    const BAR_HEIGHT: f32 = 2.;
    pub fn new(ship: Entity, module: Option<usize>) -> Self {
        Self { ship, module }
    }
}

pub fn spawn_ship_health_bars(
    mut commands: Commands,
    added_icons: Query<(&ChildOf, Entity), Added<MapIcon>>,
) {
    for (childof, icon_entity) in added_icons {
        let entity = childof.parent();
        let total_height = 4. * FighterHealthBarFill::BAR_HEIGHT + 8.;
        let top = total_height / 2. - FighterHealthBarFill::BAR_HEIGHT / 2.;
        for index in 0..4 {
            let module = if index == 0 { None } else { Some(index - 1) };
            let y = top - index as f32 * (FighterHealthBarFill::BAR_HEIGHT + 2.);

            commands.spawn((
                Sprite::from_color(
                    if module.is_none() {
                        Color::srgb(0.2, 0.55, 1.)
                    } else {
                        Color::srgb(0.2, 0.88, 0.33)
                    },
                    Vec2::new(
                        FighterHealthBarFill::BAR_WIDTH,
                        FighterHealthBarFill::BAR_HEIGHT,
                    ),
                ),
                Transform::from_xyz(0., y as f32, 3.),
                FighterHealthBarFill::new(icon_entity, module),
                ChildOf(entity),
            ));
        }
    }
}

pub fn update_health_bars(
    icons: Query<(&mut MapIcon, &ChildOf)>,
    healthbars: Query<(&FighterHealthBarFill, &mut Sprite, &mut Transform)>,
) {
    for (bar, mut sprite, mut transform) in healthbars {
        if let Ok((icon, _)) = icons.get(bar.ship) {
            let health = icon.get_health();
            let max_health = icon.get_max_health();

            let (current, max) = match bar.module {
                None => (icon.get_shields(), icon.get_max_shields()),
                Some(n) => match n {
                    0 => (health.core, max_health),
                    1 => (health.engines, max_health),
                    2 => (health.weapons, max_health),
                    _ => {
                        continue;
                    }
                },
            };
            let ratio = if max > 0 {
                (current as f32 / max as f32).clamp(0., 1.)
            } else {
                0.
            };

            let bar_width = ratio * FighterHealthBarFill::BAR_WIDTH;
            sprite.custom_size = Some(Vec2::new(bar_width, FighterHealthBarFill::BAR_HEIGHT));

            transform.translation.x = -FighterHealthBarFill::BAR_WIDTH / 2. + bar_width / 2.;

            sprite.color = if bar.module.is_none() {
                Color::srgb(0.2, 0.55, 1.)
            } else if ratio > 0.6 {
                Color::srgb(0.2, 0.88, 0.33)
            } else if ratio > 0.25 {
                Color::srgb(1.0, 0.33, 0.88)
            } else {
                Color::srgb(1.0, 0.2, 0.2)
            };
        }
    }
}

// ##################################################### # # # ATTACKING # # # ######################################################

#[derive(Component, Debug)]
pub struct Bullet;

#[derive(Resource, Debug)]
pub struct ShootCooldownTimer(pub Timer);

pub fn shoot(
    commands: &mut Commands,
    destination: DVec2,
    shooter_transform: &Transform,
    cooldown: &mut ResMut<ShootCooldownTimer>,
    time: &Res<Time>,
) {
    cooldown.0.tick(time.delta());
    if cooldown.0.just_finished() {
        let forward = shooter_transform.rotation * Vec3::Y;
        let start_bullet_loc = shooter_transform.translation + forward * 72.;
        let to_target = destination - start_bullet_loc.truncate().as_dvec2();
        cooldown.0.reset();
        cooldown.0.unpause();
        if to_target.dot(forward.truncate().as_dvec2()) <= 0. {
            return;
        }
        let dir = to_target
            .normalize()
            .as_vec2();
        let mut transform = Transform::from_translation(start_bullet_loc);
        transform.rotation = Quat::from_rotation_z(dir.y.atan2(dir.x));
        commands.spawn((
            Sprite::from_color(Color::srgb(1., 1., 1.), Vec2::new(3., 10.)),
            transform,
            bevy_rapier2d::dynamics::Velocity::linear(dir * 2733.),
            Collider::cuboid(1.5, 5.),
            Ccd::enabled(),
            ActiveEvents::COLLISION_EVENTS,
            CollisionGroups::new(Group::GROUP_2, Group::GROUP_1),
            RigidBody::Dynamic,
            GravityScale(0.),
            Bullet {},
        ));
    }
}

// #################################################### # # # BULLET LOGIC # # # ####################################################

pub fn ship_bullet_collision(
    mut commands: Commands,
    mut collision_events: MessageReader<CollisionEvent>,
    bullet_query: Query<&Transform, With<Bullet>>,
    mut ship_query: Query<(&ChildOf, &mut MapIcon)>,
    ship_t_query: Query<&Transform, Without<Bullet>>,
    children: Query<&Children>,
) {
    for event in collision_events.read() {
        match event {
            CollisionEvent::Started(h1, h2, _flags) => {
                let entity1_is_bullet = bullet_query.get(*h1).is_ok();
                let entity2_is_bullet = bullet_query.get(*h2).is_ok();

                if let Some(entity1) = children
                    .get(*h1)
                    .ok()
                    .and_then(|children| children.iter().find(|child| ship_query.contains(*child)))
                {
                    let entity1_is_ship = ship_query.get(entity1).is_ok();
                    if entity1_is_ship && entity2_is_bullet {
                        if let Ok((icon_parent, mut icon)) = ship_query.get_mut(entity1) {
                            if let Ok(transform) = bullet_query.get(*h2) {
                                if let Ok(ship_transform) = ship_t_query.get(icon_parent.parent()) {
                                    let target_shields = icon.get_shields();
                                    let mut part = which_part_to_injure(
                                        icon.get_pos(),
                                        transform.translation.truncate().as_dvec2(),
                                        ship_transform,
                                        target_shields,
                                    );
                                    let mut ship_health = icon.get_health();
                                    if ship_health.are_engines_wrecked() && part == "engines" {
                                        part = "core";
                                    } else if ship_health.are_weapons_wrecked() && part == "weapons"
                                    {
                                        part = "core";
                                    }
                                    icon.injure(400, part);
                                    ship_health = icon.get_health();
                                    if ship_health.is_ship_wrecked() {
                                        commands.entity(entity1).queue_silenced(despawn());
                                    }
                                }
                            }
                        }
                        commands.entity(*h2).queue_silenced(despawn());
                    }
                }
                if let Some(entity2) = children
                    .get(*h2)
                    .ok()
                    .and_then(|children| children.iter().find(|child| ship_query.contains(*child)))
                {
                    let entity2_is_ship = ship_query.get(entity2).is_ok();
                    if entity1_is_bullet && entity2_is_ship {
                        if let Ok((icon_parent, mut icon)) = ship_query.get_mut(entity2) {
                            if let Ok(transform) = bullet_query.get(*h1) {
                                if let Ok(ship_transform) = ship_t_query.get(icon_parent.parent()) {
                                    let target_shields = icon.get_shields();
                                    let mut part = which_part_to_injure(
                                        icon.get_pos(),
                                        transform.translation.truncate().as_dvec2(),
                                        ship_transform,
                                        target_shields,
                                    );
                                    let mut ship_health = icon.get_health();
                                    if ship_health.are_engines_wrecked() && part == "engines" {
                                        part = "core";
                                    } else if ship_health.are_weapons_wrecked() && part == "weapons"
                                    {
                                        part = "core";
                                    }
                                    icon.injure(400, part);
                                    ship_health = icon.get_health();
                                    if ship_health.is_ship_wrecked() {
                                        commands.entity(entity2).queue_silenced(despawn());
                                    }
                                }
                            }
                        }
                        commands.entity(*h1).queue_silenced(despawn());
                    }
                }
            }
            CollisionEvent::Stopped(_, _, _) => {}
        }
    }
}
