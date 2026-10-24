use std::{
    f32::consts::{PI, TAU},
    f64::consts::FRAC_PI_2,
};

use bevy::{
    camera::Camera2d,
    ecs::{
        resource::Resource,
        system::entity_command::{despawn, remove},
    },
    input::{
        ButtonInput,
        keyboard::KeyCode,
        mouse::{AccumulatedMouseMotion, MouseWheel},
    },
    math::DVec2,
    prelude::*,
    window::PrimaryWindow,
};

use crate::{
    npcs::ShipAiCooldownTimer,
    ships::{
        FighterHealth, ShipType, ShootCooldownTimer, shoot, spawn_ship, update_fighter_texture,
    },
};

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        // app.add_plugins(InfiniteGridPlugin);
        app.add_systems(
            FixedUpdate,
            (update_map_camera, update_camera_from_map_camera).chain(),
        );
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub enum Factions {
    TestAlly,
    TestEnemy,
    Player,
}

impl Factions {
    pub fn get_color(&self) -> Color {
        match self {
            Self::TestAlly => Color::srgb(0.5, 0.33, 0.01),
            Self::TestEnemy => Color::srgb(0.77, 0.33, 0.67),
            Self::Player => Color::srgb(0.1, 1., 0.1),
        }
    }
}

#[derive(Component, Clone, Debug, PartialEq)]
pub enum IconType {
    Ship(ShipType),
}

#[derive(Debug, Component, Clone)]
pub enum RelationType {
    Enemy,
    Ally,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, States)]
pub enum MapState {
    #[default]
    Universe,
    Sector(u16),
}

#[derive(Resource, Debug)]
pub struct MapCamera {
    pub center: DVec2,
    pub zoom: f64,
    pub viewport_size: Vec2,
}

impl MapCamera {
    pub fn update_pos_from_vec2(&mut self, vec: &Vec2) {
        let dvec = DVec2 {
            x: vec.x as f64,
            y: vec.y as f64,
        };
        self.center += dvec;
    }
}

#[derive(Component, Debug, Clone)]
pub struct MapIcon {
    // pub size: f32,
    pub icon_type: IconType,
    pub relations_type: RelationType,
}

impl MapIcon {
    pub fn new(icon_type: IconType, relations_type: RelationType) -> Self {
        Self {
            icon_type,
            relations_type,
        }
    }

    pub fn get_pos(&self) -> DVec2 {
        match &self.icon_type {
            IconType::Ship(ship) => match ship {
                ShipType::Fighter(fighter) => fighter.pos,
            },
        }
    }

    pub fn get_health(&self) -> FighterHealth {
        match &self.icon_type {
            IconType::Ship(ship) => match ship {
                ShipType::Fighter(fighter) => fighter.health.clone(),
            },
        }
    }

    pub fn get_max_health(&self) -> i32 {
        match &self.icon_type {
            IconType::Ship(ship) => match ship {
                ShipType::Fighter(fighter) => fighter.max_health.clone(),
            },
        }
    }

    pub fn get_damage(&self) -> i32 {
        match &self.icon_type {
            IconType::Ship(ship) => match ship {
                ShipType::Fighter(fighter) => fighter.damage,
            },
        }
    }

    pub fn update_pos(&mut self, amount: DVec2) {
        match &mut self.icon_type {
            IconType::Ship(ship) => match ship {
                ShipType::Fighter(fighter) => fighter.pos += amount,
            },
        };
    }

    pub fn get_shields(&self) -> i32 {
        match &self.icon_type {
            IconType::Ship(ship) => match ship {
                ShipType::Fighter(fighter) => fighter.shields,
            },
        }
    }

    pub fn get_max_shields(&self) -> i32 {
        match &self.icon_type {
            IconType::Ship(ship) => match ship {
                ShipType::Fighter(fighter) => fighter.max_shields,
            },
        }
    }

    pub fn injure(&mut self, amount: i32, part: &'static str) {
        match &mut self.icon_type {
            IconType::Ship(ship) => match ship {
                ShipType::Fighter(fighter) => match part.to_ascii_lowercase().as_str() {
                    "engines" => fighter.health.injure_engines(amount),
                    "core" => fighter.health.injure_core(amount),
                    "weapons" => fighter.health.injure_weapon_module(amount),
                    "shields" => fighter.shields -= amount,
                    other => println!("Didn't update, {other} part doesn't exist!"),
                },
            },
        }
    }

    pub fn get_max_vel(&self) -> f64 {
        match &self.icon_type {
            IconType::Ship(ship) => match ship {
                ShipType::Fighter(fighter) => fighter.max_vel,
            },
        }
    }

    pub fn limit_max_vel(&mut self) {
        match &mut self.icon_type {
            IconType::Ship(ship) => match ship {
                ShipType::Fighter(fighter) => fighter.max_vel = fighter.max_speed / 1.5,
            },
        }
    }

    pub fn lift_max_vel_limit(&mut self) {
        match &mut self.icon_type {
            IconType::Ship(ship) => match ship {
                ShipType::Fighter(fighter) => fighter.max_vel = fighter.max_speed,
            },
        }
    }

    pub fn get_max_speed(&self) -> f64 {
        match &self.icon_type {
            IconType::Ship(ship) => match ship {
                ShipType::Fighter(fighter) => fighter.max_speed,
            },
        }
    }

    pub fn get_acceleration(&self) -> f64 {
        match &self.icon_type {
            IconType::Ship(ship) => match ship {
                ShipType::Fighter(fighter) => fighter.acceleration,
            },
        }
    }

    pub fn get_color(&self) -> Color {
        match self.relations_type {
            RelationType::Enemy => Color::srgb(1., 0.33, 0.33),
            RelationType::Ally => Color::srgb(0.3, 0.7, 1.),
        }
    }

    pub fn get_line_color(&self) -> Color {
        match self.relations_type {
            RelationType::Enemy => Color::srgba(1., 0.33, 0.33, 0.67),
            RelationType::Ally => Color::srgba(0.3, 0.7, 1., 0.7),
        }
    }

    pub fn get_faction(&self) -> Factions {
        match &self.icon_type {
            IconType::Ship(ship) => match ship {
                ShipType::Fighter(fighter) => fighter.owner,
            },
        }
    }
}

#[derive(Component)]
pub struct MapVisible;

// ##################################################################### custom velocity cuz why not

#[derive(Component, Debug)]
pub struct Velocity {
    pub speed: f64,
}

pub fn update_pos_from_velocity(
    mut vel_transform_query: Query<(&mut Velocity, &mut Transform, &Children)>,
    mut children_q: Query<&mut MapIcon>,
) {
    for (vel, mut transform, children) in &mut vel_transform_query {
        let dir = transform.rotation * Vec3::Y;
        let linvel: DVec2;
        {
            let vel_x = dir.x as f64 * vel.speed;
            let vel_y = dir.y as f64 * vel.speed;
            linvel = DVec2::new(vel_x, vel_y)
        }

        for child in children {
            if let Ok(mut icon) = children_q.get_mut(*child) {
                icon.update_pos(linvel);

                transform.translation += linvel.extend(0.).as_vec3();
            }
        }
    }
}

impl Velocity {
    pub const ZERO: Self = Self { speed: 0. };

    pub fn add_from_endpoint(&mut self, max_speed: f64, acceleration: f64, time: &Res<Time>) {
        if self.speed == max_speed {
            return;
        }
        self.speed += acceleration * time.delta_secs_f64();

        if self.speed > max_speed {
            self.speed = max_speed;
        }
    }

    pub fn brake(&mut self, acceleration: f64, time: &Res<Time>, max_vel: f64) {
        let new_speed = (self.speed - 4.2 * acceleration * time.delta_secs_f64()).max(0.);

        if (self.speed > 0.) && !(self.speed < max_vel / 10.) {
            self.speed = new_speed;
        }
    }
}

// ##################################################################### minor update functions.

pub fn update_ship_pos(icon_q: Query<(&mut MapIcon, &ChildOf)>, transform_q: Query<&Transform>) {
    for (mut icon, child_of) in icon_q {
        let Ok(transform) = transform_q.get(child_of.parent()) else {
            continue;
        };

        match &mut icon.icon_type {
            IconType::Ship(ship) => match ship {
                ShipType::Fighter(fighter) => {
                    fighter.pos = transform.translation.truncate().as_dvec2()
                }
            },
        }
    }
}

// ##################################################################### some minor markers and other stuff (bevy)

#[derive(Component, Debug)]
pub struct Rotated;

// ##################################################################### some minor helper functions

pub fn check_if_clicked_inside_an_object(
    object_center: DVec2,
    click: DVec2,
    add_x: f64,
    add_y: f64,
) -> bool {
    let condition_left = object_center.x - add_x < click.x;
    let condition_right = object_center.x + add_x > click.x;
    let condition_down = object_center.y - add_y < click.y;
    let condition_up = object_center.y + add_y > click.y;

    condition_down && condition_up && condition_left && condition_right
}

pub fn is_close_enough(pos1: DVec2, pos2: DVec2, offset: f64) -> bool {
    pos1.distance(pos2) <= offset
}

pub fn which_part_to_injure(
    def_pos: DVec2,
    atk_pos: DVec2,
    def_t: &Transform,
    shields: i32,
) -> &'static str {
    if shields > 0 {
        return "shields";
    }
    let direction = (atk_pos - def_pos).normalize_or_zero();

    let y_target = def_t.local_y().as_dvec3().truncate();
    let x_target = def_t.local_x().as_dvec3().truncate();

    let y_align = direction.dot(y_target);
    let x_align = direction.dot(x_target);
    if y_align.abs() > x_align.abs() {
        if y_align > 0. { "core" } else { "engines" }
    } else {
        "weapons"
    }
}

// ##################################################################### updates the camera.

pub fn update_camera_from_map_camera(
    map_camera: Res<MapCamera>,
    camera_q: Single<(&mut Transform, &mut Projection), With<Camera2d>>,
) {
    let (mut camera_t, mut projection) = camera_q.into_inner();
    camera_t.translation.x = map_camera.center.x as f32;
    camera_t.translation.y = map_camera.center.y as f32;

    if let Projection::Orthographic(orto_proj) = &mut *projection {
        orto_proj.scale = 1.0 / map_camera.zoom as f32
    }
}

pub fn update_map_camera(
    // keyboard: Res<ButtonInput<KeyCode>>, // preserved for future
    mouse: Res<ButtonInput<MouseButton>>,
    mut map_camera: ResMut<MapCamera>,
    mut mouse_wheel: MessageReader<MouseWheel>,
    mouse_motion: Res<AccumulatedMouseMotion>,
) {
    for event in mouse_wheel.read() {
        if map_camera.zoom > 0.4 && map_camera.zoom < 20. {
            map_camera.zoom += event.y as f64 / 5.; // update the map zoom by the amount was scrolled.
        } else if map_camera.zoom <= 0.4 {
            if event.y >= 0. {
                map_camera.zoom += event.y as f64 / 5.; // update the map zoom by the amount was scrolled IF IT'S ZOOMING IN.
            }
        } else if map_camera.zoom > 10. {
            if event.y <= 0. {
                map_camera.zoom += event.y as f64 / 5.; // update the map zoom by the amount was scrolled IF IT'S ZOOMING OUT.
            }
        }
        // Saved for future development.
        // match event.unit {
        //     bevy::input::mouse::MouseScrollUnit::Line => {},
        //     bevy::input::mouse::MouseScrollUnit::Pixel => {}
        // }
    }

    if mouse.pressed(MouseButton::Right) {
        let mut delta_mouse_motion = mouse_motion.delta; // get mouse delta from the last frame

        delta_mouse_motion.x = -delta_mouse_motion.x;

        map_camera.update_pos_from_vec2(&delta_mouse_motion); // update the map's centre pos
    }
}

// ################################################################### calculates positions

// pub fn map_to_screen(
//     // from map (simulation) to screen (visualisation)
//     map_pos: DVec2,
//     camera: &MapCamera,
// ) -> Vec2 {
//     let mut pos = map_pos.clone();
//
//     pos -= camera.center;
//
//     pos.x *= camera.zoom;
//     pos.y *= -camera.zoom;
//
//     pos.x += camera.viewport_size.x as f64 / 2.;
//     pos.y += camera.viewport_size.y as f64 / 2.;
//
//     Vec2 {
//         x: pos.x as f32,
//         y: pos.y as f32,
//     }
// }

pub fn screen_to_map(
    // from screen (visualisation) to map (simulation)
    screen_pos: Vec2,
    camera: &MapCamera,
) -> DVec2 {
    let mut pos = DVec2 {
        x: screen_pos.x as f64,
        y: screen_pos.y as f64,
    };

    pos.x -= camera.viewport_size.x as f64 / 2.;
    pos.y -= camera.viewport_size.y as f64 / 2.;

    pos.x /= camera.zoom;
    pos.y /= -camera.zoom;

    pos += camera.center;

    pos
}

// ######################################################################### Level of Detail:
// This will be useful later on (probably)

// pub struct MapLoD {
//     pub show_invidual_ships: bool,
//     pub show_fleets: bool,
//     pub show_station_labels: bool,
//     pub show_ship_labels: bool,
//     pub show_routes: bool,
// }
//
// pub fn calculate_lod(camera: Res<MapCamera>) -> MapLoD {
//     let mut show_invidual_ships = false;
//     let mut show_fleets = false;
//     let mut show_station_labels = false;
//     let mut show_ship_labels = false;
//     let show_routes = true; // This is ALWAYS true.
//
//     if camera.zoom > 0. {
//         if camera.zoom < 1. {
//             show_invidual_ships = true;
//             show_station_labels = true;
//             show_ship_labels = true;
//         } else if camera.zoom < 5. {
//             show_fleets = true;
//             show_station_labels = true;
//         } else {
//             show_fleets = true;
//         }
//     }
//     MapLoD {
//         show_invidual_ships,
//         show_fleets,
//         show_station_labels,
//         show_ship_labels,
//         show_routes,
//     }
// }

// ##################################################### # # # RENDERING # # # ######################################################

// ############## Routes

#[derive(Component, Clone, Debug, Default, PartialEq, Eq)]
pub enum Order {
    #[default]
    None,
    Fly,
    Attack(Entity),
}

#[derive(Component, Clone, Debug)]
pub struct MapRoute {
    pub path_endpoints: Vec<DVec2>,
}

impl MapRoute {
    pub fn new(start: DVec2, end: DVec2) -> Self {
        Self {
            path_endpoints: vec![start, end],
        }
    }
}

pub fn render_routes(mut gizmos: Gizmos, route_q: Query<(&MapRoute, &MapIcon), With<Order>>) {
    for (route, icon) in route_q {
        if let Some(start) = route.path_endpoints.first() {
            if let Some(end) = route.path_endpoints.last() {
                let player_pos = icon.get_pos();
                let line_color = icon.get_line_color(); // this is the line indicating the part of the route that was done by now
                let line_color_left = icon.get_color(); // this is the line indicating the part of the route that was NOT done by now
                gizmos.line_2d(start.as_vec2(), player_pos.as_vec2(), line_color);
                gizmos.line_2d(player_pos.as_vec2(), end.as_vec2(), line_color_left);
            }
        }
    }
}

// ############ Sectors

#[derive(Component, Clone)]
pub struct Sector {
    pub center: DVec2,
    pub radius: f64,
}

pub fn spawn_sector(mut commands: Commands) {
    // It's here, because it's always the same. It spawns when in MapState::Sector(_), the ID does nothing here.
    let sectors = vec![(
        Sector {
            center: DVec2::new(0., 0.),
            radius: 6700.,
        },
        Factions::TestAlly,
    )];
    for (sector, owner) in sectors {
        commands.spawn((
            sector.clone(),
            owner.clone(),
            Transform::from_translation(sector.center.as_vec2().extend(0.).clone()),
        ));
    }
}

pub fn draw_sector(mut gizmos: Gizmos, sector_q: Query<(&Sector, &Factions)>) {
    for (sector, owner) in sector_q {
        gizmos.circle_2d(
            Isometry2d::from_xy(sector.center.x as f32, sector.center.y as f32),
            sector.radius as f32,
            owner.get_color(),
        );
    }
}

// ############################################ orders and selections - the shared part ###########################################:

#[derive(Resource, Debug)]
pub struct RemoveSelection;

// ##################################################### # # # ORDERS # # # ######################################################

//temp
#[derive(Resource)]
pub struct Unpaused;
#[derive(Resource)]
pub struct Paused;

pub fn toggle_pause(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    is_unpaused: Option<Res<Unpaused>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        if is_unpaused.is_some() {
            commands.remove_resource::<Unpaused>();
            commands.insert_resource(Paused);
        } else {
            commands.insert_resource(Unpaused);
            commands.remove_resource::<Paused>();
        }
    }
}
//temp

// making orders!!1!!
pub fn order(
    mut commands: Commands,
    selected_q: Query<(Entity, &MapIcon, &ChildOf), (With<MapIconSelected>, Without<Order>)>,
    other_icons: Query<(Entity, &MapIcon), Without<MapIconSelected>>,
    cursor: Res<MapCursor>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut vel_q: Query<&mut Velocity>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        for (entity, icon, child_of) in selected_q {
            let pos = icon.get_pos();

            if let Ok(mut vel) = vel_q.get_mut(child_of.0) {
                vel.speed = 0.;
            }

            for (entity2, icon2) in other_icons {
                let icon2_pos = icon2.get_pos();
                if check_if_clicked_inside_an_object(icon2_pos, cursor.pos, 20., 20.) {
                    commands
                        .entity(entity)
                        .insert((MapRoute::new(pos, cursor.pos), Order::Attack(entity2)));
                    println!("Attacking {icon:?}");
                    commands.insert_resource(RemoveSelection);
                    return;
                }
            }

            commands
                .entity(entity)
                .insert((MapRoute::new(pos, cursor.pos), Order::Fly));
            commands.insert_resource(RemoveSelection);
        }
    }
}

// Fulfilling orders
pub fn fulfill_orders(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut orders: Query<(
        Entity,
        Option<&mut Order>,
        Option<&mut MapRoute>,
        &mut MapIcon,
        &ChildOf,
    )>,
    mut map_icons: Query<(&mut Velocity, Option<&Rotated>)>,
    mut transform_q: Query<&mut Transform>,
    time: Res<Time>,
    ai_cooldown: ResMut<ShipAiCooldownTimer>,
    mut attack_cooldown: ResMut<ShootCooldownTimer>,
    _unpaused: If<Res<Unpaused>>,
) {
    let mut attack_vec: Vec<[Entity; 2]> = vec![];
    for (entity, order, route, icon, child_of) in &orders {
        if route.is_none() && order.is_none() {
            let acceleration = icon.get_acceleration();
            let max_vel = icon.get_max_vel();
            if let Ok((mut vel, _)) = map_icons.get_mut(child_of.parent()) {
                vel.brake(acceleration, &time, max_vel);
            }
        }
        match order {
            Some(Order::Attack(target)) => {
                attack_vec.push([entity, *target]);
            }
            None => continue,
            _ => {}
        }
        if icon.get_health().are_engines_wrecked() {
            continue;
        }
        if let Some(route) = route {
            if let Some(endpoint) = route.path_endpoints.last() {
                let parent = child_of.parent();
                let pos = icon.get_pos();
                let acceleration = icon.get_acceleration();

                if let Ok(mut transform) = transform_q.get_mut(parent) {
                    let dir = endpoint - pos;

                    let target_angle = (dir.y.atan2(dir.x) - FRAC_PI_2) as f32;

                    
                    let (_, _, current_angle) = transform.rotation.to_euler(EulerRot::XYZ);

                    let angle_diff = (target_angle - current_angle + PI).rem_euclid(TAU) - PI;

                    let turn_speed = 240_f32.to_radians();
                    let max_turn = turn_speed * time.delta_secs();

                    let turn = angle_diff.clamp(-max_turn, max_turn);
                    transform.rotate_z(turn);
                    
                    if angle_diff.abs() <= max_turn {
                        transform.rotation = Quat::from_rotation_z(target_angle);
                        commands.entity(parent).insert(Rotated);
                    } else {
                        commands.entity(parent).remove::<Rotated>();
                    }
                }
                if let Ok((mut vel, rotated)) = map_icons.get_mut(parent) {
                    let max_vel = icon.get_max_vel();
                    let distance_to_finish = (endpoint - pos).length();

                    // and the fun part!
                    let current_speed = vel.speed;

                    if distance_to_finish < 1. {
                        vel.speed = 0.;
                        commands.entity(parent).queue_silenced(remove::<Rotated>());
                        commands.entity(entity).queue_silenced(remove::<MapRoute>());
                        if order == Some(&Order::Fly) {
                            commands.entity(entity).queue_silenced(remove::<Order>());
                        }
                    } else if distance_to_finish <= current_speed as f64 * 42. {
                        if !(matches!(order, Some(&Order::Attack(_)))) {
                            vel.brake(acceleration, &time, max_vel);
                        }
                    } else {
                        if rotated.is_none() {
                            vel.add_from_endpoint(max_vel, acceleration / 5., &time);
                        } else {
                            vel.add_from_endpoint(max_vel, acceleration, &time);
                        }
                    }
                }
            }
        }
    }
    for entity_array in attack_vec.clone() {
        if let Ok(
            [
                (base_entity, _base_order, base_route, mut base_icon, base_child_of),
                (target_entity, _target_order, target_route, mut target_icon, target_child_of),
            ],
        ) = orders.get_many_mut(entity_array)
        {
            
            let base_health = base_icon.get_health();

            let base_pos = base_icon.get_pos();
            let target_pos = target_icon.get_pos();
            if let Some(timer) = ai_cooldown.0.get(&base_entity) {
                if timer.is_finished() {
                    commands
                        .entity(base_entity)
                        .queue_silenced(remove::<MapRoute>())
                        .insert(MapRoute::new(base_pos.clone(), target_pos.clone()));
                }
            }
            if base_route.is_none() && target_route.is_some() {
                commands
                    .entity(base_entity)
                    .insert(MapRoute::new(base_pos.clone(), target_pos.clone()));
            } else if base_route.is_none() && !is_close_enough(base_pos, target_pos, 1000.) {
                commands
                    .entity(base_entity)
                    .insert(MapRoute::new(base_pos.clone(), target_pos.clone()));
            // } else if base_route.is_some() && is_close_enough(base_pos, target_pos, 100.) {
            //     commands.entity(base_entity).remove::<MapRoute>();
            } else if is_close_enough(base_pos, target_pos, 700.) {
                if let Ok(transform) = transform_q.get(base_child_of.parent()) {
                    if !base_health.are_weapons_wrecked() {
                        shoot(
                            &mut commands,
                            target_pos,
                            transform,
                            &mut attack_cooldown,
                            &time,
                        );
                    }
                    let forward = (transform.rotation * Vec3::Y).truncate();
                    let to_target = (target_pos - base_pos).as_vec2().normalize_or_zero();
                    let facing = forward.dot(to_target);
                    if facing <= 0.3 {
                        base_icon.limit_max_vel();
                    } else {
                        base_icon.lift_max_vel_limit();
                    }
                }

                let target_health = target_icon.get_health();
                if target_health.is_ship_wrecked() {
                    commands.entity(target_entity).queue_silenced(despawn());
                    commands.entity(target_child_of.0).queue_silenced(despawn());
                    commands
                        .entity(base_entity)
                        .queue_silenced(remove::<Order>())
                        .queue_silenced(remove::<MapRoute>())
                        .queue_silenced(remove::<Rotated>());
                    if let Ok((mut vel, _)) = map_icons.get_mut(base_entity) {
                        vel.speed = 0.;
                    }
                } else {
                    let wreckage = target_health.estimate_wreckage();
                    update_fighter_texture(
                        &mut commands,
                        &asset_server,
                        (
                            target_child_of.parent(),
                            target_icon.get_faction(),
                            wreckage,
                        ),
                    );
                }
            }
        } else if let Ok((entity, _, _, _, _)) = orders.get_mut(entity_array[0]) {
            commands
                .entity(entity)
                .queue_silenced(remove::<Order>())
                .queue_silenced(remove::<MapRoute>())
                .queue_silenced(remove::<Rotated>());
        }
    }
}

// ##################################################### # # # SELECTIONS # # # ######################################################

#[derive(Resource, Debug)]
pub struct MapCursor {
    pub pos: DVec2,
}

#[derive(Component, Debug)]
pub struct MapIconSelected;

pub fn update_cursor_pos(
    mut cursor: ResMut<MapCursor>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera: Res<MapCamera>,
) {
    for window in windows {
        if let Some(current_cursor_pos) = window.cursor_position() {
            cursor.pos = screen_to_map(current_cursor_pos, &camera);
        }
    }
}

// ############################ selection, this needs to be improved
// pub fn select(
//     mut commands: Commands,
//     unselected_q: Query<(Entity, &MapIcon), (Without<MapIconSelected>, Without<NonPlayableShip>)>,
//     selected_q: Query<Entity, (With<MapIconSelected>, Without<NonPlayableShip>)>,
//     cursor: Res<MapCursor>,
//     mouse: Res<ButtonInput<MouseButton>>,
//     remove_trigger: Option<Res<RemoveSelection>>,
// ) {
//     if mouse.just_pressed(MouseButton::Left) {
//         if remove_trigger.is_none() {
//             if selected_q.is_empty() {
//                 for (entity, icon) in unselected_q {
//                     let icon_pos = icon.get_pos();
//                     if check_if_clicked_inside_an_object(icon_pos, cursor.pos, 20., 20.) {
//                         println!("SELECTED A SHIP.");
//                         commands.entity(entity).insert(MapIconSelected);
//                         return;
//                     }
//                 }
//             }
//         } else {
//             for entity in selected_q {
//                 commands.entity(entity).remove::<MapIconSelected>();
//             }
//             commands.remove_resource::<RemoveSelection>();
//         }
//     }
// }

// ##################################################### # # # ADDING SHIPS WHILE IN PAUSE  # # # ######################################################
pub fn add_ship_from_input(
    mut commands: Commands,
    cursor: Res<MapCursor>,
    mouse: Res<ButtonInput<MouseButton>>,
    asset_server: Res<AssetServer>,
    _paused: If<Res<Paused>>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        let image: Handle<Image> = asset_server.load("icons/fighter_ally.png");
        spawn_ship(
            &mut commands,
            1,
            cursor.pos,
            Factions::TestAlly,
            20.,
            5.3,
            3000,
            123,
            3333,
            10,
            RelationType::Ally,
            image,
        );
    } else if mouse.just_pressed(MouseButton::Middle) {
        let image: Handle<Image> = asset_server.load("icons/fighter_enemy.png");
        spawn_ship(
            &mut commands,
            1,
            cursor.pos,
            Factions::TestEnemy,
            20.,
            5.3,
            3000,
            123,
            3333,
            10,
            RelationType::Enemy,
            image,
        );
    }
}
