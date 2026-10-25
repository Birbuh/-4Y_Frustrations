//! Fighter data, ship creation, damage visuals, projectiles, and collisions.
//!
//! Each ship has a parent entity with its sprite, transform, collider, and
//! custom [`Velocity`]. Its child owns [`MapIcon`], including health and faction,
//! and receives routes and orders. Attack targets refer to that child entity.
//! Health-bar sprites are siblings of the icon under the same parent.
//!
//! Destroy the parent to remove the complete ship hierarchy.

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

use crate::{
    map::{
        Factions, IconType, MapIcon, MapRoute, MapVisible, Order, RelationType, Velocity,
        is_close_enough, which_part_to_injure,
    },
    npcs::Fleed,
};

/// Shared constructor contract for the ship types supported by the prototype.
pub trait Ship {
    /// Creates a ship with full shields and equal initial health for each module.
    ///
    /// `pos` is sector-local; `damage` is the damage carried by each projectile.
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

/// Independent module health and the configured repair amount.
///
/// A destroyed core means the ship is destroyed. Engine and weapon damage
/// are classified separately for movement, combat, and texture selection.
#[derive(Clone, Debug, PartialEq)]
pub struct FighterHealth {
    /// Remaining engine module health.
    pub engines: i32,
    /// Remaining core health; zero marks the ship as destroyed.
    pub core: i32,
    /// Remaining weapon module health.
    pub weapons: i32,
    /// Stored repair setting; automatic repair is not implemented yet.
    pub repair_speed: i32,
}

impl FighterHealth {
    /// Initializes every module to `max_health` and stores the repair setting.
    pub fn new(max_health: i32, repair_speed: i32) -> Self {
        Self {
            engines: max_health,
            core: max_health,
            weapons: max_health,
            repair_speed,
        }
    }
    /// Applies nonnegative engine damage, clamping depleted health to zero.
    pub fn injure_engines(&mut self, amount: i32) {
        if self.engines - amount < 0 {
            self.engines = 0
        } else {
            self.engines -= amount
        }
    }

    /// Applies nonnegative core damage, clamping depleted health to zero.
    pub fn injure_core(&mut self, amount: i32) {
        if self.core - amount < 0 {
            self.core = 0
        } else {
            self.core -= amount
        }
    }

    /// Applies nonnegative weapon damage, clamping depleted health to zero.
    pub fn injure_weapon_module(&mut self, amount: i32) {
        if self.weapons - amount < 0 {
            self.weapons = 0
        } else {
            self.weapons -= amount
        }
    }

    /// Returns whether engine health has reached or fallen below zero.
    pub fn are_engines_wrecked(&self) -> bool {
        self.engines <= 0
    }

    /// Returns whether weapon health has reached or fallen below zero.
    pub fn are_weapons_wrecked(&self) -> bool {
        self.weapons <= 0
    }

    /// Returns whether the core is destroyed, regardless of other module health.
    pub fn is_ship_wrecked(&self) -> bool {
        self.core <= 0
    }

    /// Classifies engine and weapon destruction for texture selection.
    ///
    /// Core destruction is handled by ship removal and is not represented here.
    pub fn estimate_wreckage(&self) -> Wreckage {
        match (self.are_engines_wrecked(), self.are_weapons_wrecked()) {
            (true, true) => Wreckage::EnginesWeapons,
            (true, false) => Wreckage::Engines,
            (false, true) => Wreckage::Weapons,
            (false, false) => Wreckage::None,
        }
    }
}

/// Queues a fighter's physical parent and logical `MapIcon` child.
///
/// The parent owns the sprite, collider, and movement components. AI and
/// attack orders use the child entity. Health bars are added separately by
/// [`spawn_ship_health_bars`] when the icon becomes visible to queries.
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

/// Sector-local fighter state stored inside [`MapIcon`].
#[derive(Component, Clone, Debug, PartialEq)]
pub struct Fighter {
    /// Current sector, or `None` when no sector is assigned.
    pub sector_id: Option<u16>,
    /// Position inside the sector, synchronized from the parent's transform.
    pub pos: DVec2,
    /// Faction used for AI hostility decisions.
    pub owner: Factions,
    /// Current movement cap, which combat steering can temporarily reduce.
    pub max_vel: f64,
    /// Unrestricted movement cap used when restoring normal speed.
    pub max_speed: f64,
    /// Acceleration used by the custom movement controller.
    pub acceleration: f64,
    // pub cargo: Vec<(ProductType, Capacity)>,
    /// Current shield health, which can fall below zero after a hit.
    pub shields: i32,
    /// Initial shield capacity used by the health display.
    pub max_shields: i32,
    /// Current health of the three ship modules.
    pub health: FighterHealth,
    /// Shared maximum health for each individual module.
    pub max_health: i32,
    /// Damage assigned to each spawned projectile.
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

/// Ship-specific state carried by a map icon.
#[derive(Clone, Debug, PartialEq)]
pub enum ShipType {
    Fighter(Fighter),
}

/// Queues an attack order against a hostile icon within 2,000 world units.
///
/// The candidate query includes disabled ships; engine damage does not
/// exclude a target. Searches stop at a hostile without an order, otherwise
/// later matching candidates can replace the selected target. Player ships
/// do not select targets through this function.
pub fn look_for_enemies(
    commands: &mut Commands,
    ship: (Entity, &MapIcon),
    other_ships: &Query<(
        Entity,
        &MapIcon,
        Option<&Fleed>,
        Option<&MapRoute>,
        Option<&Order>,
    )>,
    ship_pos: DVec2,
) {
    let (entity, icon) = ship;
    let ship_owner = icon.get_faction();
    match ship_owner {
        Factions::TestAlly => {
            for (entity2, other_icon, _, _, other_order) in other_ships {
                if entity == entity2 {
                    continue;
                }
                let other_ship_pos = other_icon.get_pos();
                if is_close_enough(ship_pos, other_ship_pos, 2000.) {
                    match other_icon.get_faction() {
                        Factions::TestEnemy => {
                            commands.entity(entity).insert(Order::Attack(entity2));
                            if other_order.is_none() {
                                break;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        Factions::TestEnemy => {
            for (entity2, other_icon, _, _, other_order) in other_ships {
                if entity == entity2 {
                    continue;
                }
                let other_ship_pos = other_icon.get_pos();
                if is_close_enough(ship_pos, other_ship_pos, 2000.) {
                    match other_icon.get_faction() {
                        Factions::Player | Factions::TestAlly => {
                            commands.entity(entity).insert(Order::Attack(entity2));
                            if other_order.is_none() {
                                break;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        Factions::Player => {} // there will be another function for this
    }
}

// #################################################### # # # HEALTH -> SCREEN # # # ####################################################

/// Engine/weapon destruction combinations used to select fighter images.
#[derive(PartialEq, Eq, Debug)]
pub enum Wreckage {
    Engines,
    Weapons,
    EnginesWeapons,
    None,
}

/// Synchronizes each allied or enemy fighter's parent image with module damage.
///
/// Changes only [`Sprite::image`], preserving tint and sizing. Intact modules
/// restore the normal image; player-owned sprites are left unchanged.
pub fn update_fighter_texture(
    assets: Res<AssetServer>,
    ship_query: Query<(&ChildOf, &MapIcon)>,
    mut sprites: Query<&mut Sprite>,
) {
    for (childof, icon) in ship_query {
        let owner = icon.get_faction();
        let wreckage = icon.get_health().estimate_wreckage();
        let image: Handle<Image> = match owner {
            Factions::TestAlly => match wreckage {
                Wreckage::Engines => assets.load("icons/fighter_ally-noengines.png"),
                Wreckage::Weapons => assets.load("icons/fighter_ally-noweapons.png"),
                Wreckage::EnginesWeapons => {
                    assets.load("icons/fighter_ally-noengines-noweapons.png")
                }
                Wreckage::None => assets.load("icons/fighter_ally.png"),
            },
            Factions::TestEnemy => match wreckage {
                Wreckage::Engines => assets.load("icons/fighter_enemy-noengines.png"),
                Wreckage::Weapons => assets.load("icons/fighter_enemy-noweapons.png"),
                Wreckage::EnginesWeapons => {
                    assets.load("icons/fighter_enemy-noengines-noweapons.png")
                }
                Wreckage::None => assets.load("icons/fighter_enemy.png"),
            },
            Factions::Player => continue,
        };

        if let Ok(mut sprite) = sprites.get_mut(childof.parent()) {
            if sprite.image != image {
                sprite.image = image;
            }
        }
    }
}

/// Links a health-bar sprite to the logical ship icon whose health it displays.
#[derive(Component)]
pub struct FighterHealthBarFill {
    /// Child entity carrying `MapIcon`, rather than the physical ship parent.
    pub ship: Entity,
    /// `None` selects shields; `Some(0/1/2)` selects core/engines/weapons.
    pub module: Option<usize>,
}

impl FighterHealthBarFill {
    const BAR_WIDTH: f32 = 10.;
    const BAR_HEIGHT: f32 = 2.;
    /// Creates a bar binding using the module indices documented on `module`.
    pub fn new(ship: Entity, module: Option<usize>) -> Self {
        Self { ship, module }
    }
}

/// Adds shield, core, engine, and weapon bars for each newly added map icon.
///
/// Bars share the ship's sprite parent, so they follow its transform and are
/// removed when the complete ship hierarchy is despawned.
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

/// Updates bar width and color from current health, keeping the left edge fixed.
///
/// Ratios are clamped to `[0, 1]`; a missing icon leaves the bar unchanged.
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

/// Projectile damage applied to whichever ship the bullet collides with.
#[derive(Component, Debug)]
pub struct Bullet(pub i32);

/// Shared firing timer observed by all ships.
///
/// The current prototype gives every eligible shooter the same firing tick;
/// this is not an individual weapon cooldown.
#[derive(Resource, Debug)]
pub struct ShootCooldownTimer(pub Timer);

/// Queues a projectile on the shared firing tick when the target is ahead.
///
/// The caller checks range and weapon health. The muzzle is 72 world units
/// along the ship's local positive Y axis; the projectile aims from that
/// position toward `destination`.
pub fn shoot(
    commands: &mut Commands,
    destination: DVec2,
    shooter_transform: &Transform,
    shooter_icon: &MapIcon,
    cooldown: &mut ResMut<ShootCooldownTimer>,
    _time: &Res<Time>,
) {
    if cooldown.0.just_finished() {
        // Ship artwork faces +Y; projectile velocity is independent of ship motion.
        let forward = shooter_transform.rotation * Vec3::Y;
        let start_bullet_loc = shooter_transform.translation + forward * 72.;
        let to_target = destination - start_bullet_loc.truncate().as_dvec2();
        if to_target.dot(forward.truncate().as_dvec2()) <= 0. {
            return;
        }
        let dir = to_target.normalize().as_vec2();
        let mut transform = Transform::from_translation(start_bullet_loc);
        transform.rotation = Quat::from_rotation_z(dir.y.atan2(dir.x));
        commands.spawn((
            Sprite::from_color(Color::srgb(1., 1., 1.), Vec2::new(3., 10.)),
            transform,
            bevy_rapier2d::dynamics::Velocity::linear(dir * 4333.),
            Collider::cuboid(1.5, 5.),
            Ccd::enabled(),
            ActiveEvents::COLLISION_EVENTS,
            CollisionGroups::new(Group::GROUP_2, Group::GROUP_1),
            RigidBody::Dynamic,
            GravityScale(0.),
            Bullet(shooter_icon.get_damage()),
        ));
    }
}

/// Advances the shared firing timer once per simulation step.
pub fn tick_shoot_cooldown(mut cooldown: If<ResMut<ShootCooldownTimer>>, time: Res<Time>) {
    cooldown.0.0.tick(time.delta());
}
/// Restarts a finished firing timer after all shooters have observed it.
pub fn reset_shoot_cooldown(mut cooldown: If<ResMut<ShootCooldownTimer>>) {
    if cooldown.0.0.is_finished() {
        cooldown.0.0.reset();
        cooldown.0.0.unpause();
    }
}
// #################################################### # # # BULLET LOGIC # # # ####################################################

/// Applies damage and consumes bullets when ship/projectile contact starts.
///
/// Rapier events identify physical parents; their `MapIcon` children contain
/// health. Further hits on a destroyed engine or weapon module damage the
/// core. A destroyed core queues removal of the complete ship hierarchy.
pub fn ship_bullet_collision(
    mut commands: Commands,
    mut collision_events: MessageReader<CollisionEvent>,
    bullet_query: Query<(&Transform, &Bullet)>,
    mut ship_query: Query<(&ChildOf, &mut MapIcon)>,
    ship_t_query: Query<&Transform, Without<Bullet>>,
    children: Query<&Children>,
) {
    for event in collision_events.read() {
        match event {
            CollisionEvent::Started(h1, h2, _flags) => {
                let entity1_is_bullet = bullet_query.get(*h1).is_ok();
                let entity2_is_bullet = bullet_query.get(*h2).is_ok();

                // Collision handles refer to collider parents, not the logical icons.
                if let Some(entity1) = children
                    .get(*h1)
                    .ok()
                    .and_then(|children| children.iter().find(|child| ship_query.contains(*child)))
                {
                    let entity1_is_ship = ship_query.get(entity1).is_ok();
                    if entity1_is_ship && entity2_is_bullet {
                        if let Ok((icon_parent, mut icon)) = ship_query.get_mut(entity1) {
                            if let Ok((transform, bullet)) = bullet_query.get(*h2) {
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
                                    icon.injure(bullet.0, part);
                                    ship_health = icon.get_health();
                                    if ship_health.is_ship_wrecked() {
                                        // Remove the parent too, including sprite, collider, and bars.
                                        commands.entity(entity1).queue_silenced(despawn());
                                        commands
                                            .entity(icon_parent.parent())
                                            .queue_silenced(despawn());
                                    }
                                }
                            }
                        }
                        commands.entity(*h2).queue_silenced(despawn());
                    }
                }
                // Rapier can report the same pairing with the collider order reversed.
                if let Some(entity2) = children
                    .get(*h2)
                    .ok()
                    .and_then(|children| children.iter().find(|child| ship_query.contains(*child)))
                {
                    let entity2_is_ship = ship_query.get(entity2).is_ok();
                    if entity1_is_bullet && entity2_is_ship {
                        if let Ok((icon_parent, mut icon)) = ship_query.get_mut(entity2) {
                            if let Ok((transform, bullet)) = bullet_query.get(*h1) {
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
                                    icon.injure(bullet.0, part);
                                    ship_health = icon.get_health();
                                    if ship_health.is_ship_wrecked() {
                                        // Remove the parent too, including sprite, collider, and bars.
                                        commands.entity(entity2).queue_silenced(despawn());
                                        commands
                                            .entity(icon_parent.parent())
                                            .queue_silenced(despawn());
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
