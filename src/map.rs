use std::{
    default,
    f32::consts::{PI, TAU},
    f64::consts::FRAC_PI_2,
    process::Child,
};

use bevy::{
    camera::Camera2d,
    dev_tools::{
        diagnostics_overlay::DiagnosticsOverlayStatistic,
        infinite_grid::{InfiniteGrid, InfiniteGridPlugin, InfiniteGridSettings},
    },
    ecs::resource::Resource,
    input::{
        ButtonInput,
        keyboard::{Key::ColorF2Yellow, KeyCode},
        mouse::{AccumulatedMouseMotion, MouseWheel},
    },
    math::{DVec2, VectorSpace},
    mesh::PrimitiveTopology::{LineList, LineStrip},
    prelude::*,
    reflect::tuple_struct::TupleStructFieldIter,
    ui::Selected,
    window::PrimaryWindow,
};

use crate::ships::{Fighter, Ship, ShipType};

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
    Unfriendly,
    Neutral,
    Friendly,
    Ally,
    Player,
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

#[derive(Resource, Debug)]
pub struct MapViewport {
    pub width: f32,
    pub height: f32,
}

#[derive(Component)]
pub struct EntityPosInASector {
    pub sector: u16,
    pub pos: DVec2,
}

#[derive(Component, Debug)]
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

    pub fn update_pos(&mut self, amount: DVec2) {
        match &mut self.icon_type {
            IconType::Ship(ship) => match ship {
                ShipType::Fighter(fighter) => fighter.pos += amount,
            },
        };
    }

    pub fn get_max_vel(&self) -> f32 {
        match &self.icon_type {
            IconType::Ship(ship) => match ship {
                ShipType::Fighter(fighter) => fighter.max_vel,
            },
        }
    }

    pub fn get_acceleration(&self) -> f32 {
        match &self.icon_type {
            IconType::Ship(ship) => match ship {
                ShipType::Fighter(fighter) => fighter.acceleration,
            },
        }
    }

    pub fn get_color(&self) -> Color {
        match self.relations_type {
            RelationType::Enemy => Color::srgb(1., 0.33, 0.33),
            RelationType::Unfriendly => Color::srgb(1., 0.55, 0.55),
            RelationType::Neutral => Color::srgb(1., 1., 0.),
            RelationType::Friendly => Color::srgb(0.77, 0.77, 1.),
            RelationType::Ally => Color::srgb(0.3, 0.7, 1.),
            RelationType::Player => Color::srgb(0.2, 1., 0.2),
        }
    }

    pub fn get_line_color(&self) -> Color {
        match self.relations_type {
            RelationType::Enemy => Color::srgba(1., 0.33, 0.33, 0.67),
            RelationType::Unfriendly => Color::srgba(1., 0.55, 0.55, 0.67),
            RelationType::Neutral => Color::srgba(1., 1., 0., 0.7),
            RelationType::Friendly => Color::srgba(0.77, 0.77, 1., 0.7),
            RelationType::Ally => Color::srgba(0.3, 0.7, 1., 0.7),
            RelationType::Player => Color::srgba(0.2, 1., 0.2, 0.7),
        }
    }
}

#[derive(Component)]
pub struct MapLabel {
    pub text: String,
    pub priority: u8,
}

#[derive(Component)]
pub struct MapSelectable {
    pub selection_radius: f64,
}

#[derive(Component)]
pub struct MapVisible;

#[derive(Resource)]
pub struct MapView {
    pub viewed_sector: Entity,
}

#[derive(Resource)]
pub struct MapSelection {
    pub hovered: Option<Entity>,
    pub selected: Vec<Entity>,
}

#[derive(Debug)]
pub struct VisibleMapObject {
    pub entity: Entity,
    pub map_pos: DVec2,
    pub screen_pos: Vec2,
    pub obj_type: IconType,
    pub faction: Factions,
    pub sector_id: u16,
    pub relationship: RelationType,
}

impl VisibleMapObject {
    pub fn draw(
        &mut self,
        commands: &mut Commands,
        asset_server: &AssetServer,
        faction: Factions,
        sector_id: u16,
        relation_type: RelationType,
    ) {
        let ship_path = match relation_type {
            RelationType::Enemy => "icons/fighter_enemy.png",
            RelationType::Ally => "icons/fighter_ally.png",
            RelationType::Player => "icons/fighter_player.png",
            sth => {
                println!("oops! {sth:?} isn't implemented yet!");
                return;
            }
        };
        let image: Handle<Image> = match &self.obj_type {
            IconType::Ship(ship_type) => match ship_type {
                ShipType::Fighter(_) => asset_server.load(ship_path),
            },
        };
        commands.entity(self.entity).insert((
            // Transform::from_translation(self.map_pos.as_vec2().extend(0.)),
            // Velocity::ZERO,
            MapSelectable {
                selection_radius: 33.5,
            },
            Sprite::from_image(image),
        ));
    }
}

// ##################################################################### custom velocity cuz why not

#[derive(Component, Debug)]
pub struct Velocity {
    linvel: Vec2,
}

pub fn update_pos_from_velocity(
    mut vel_transform_query: Query<(&mut Velocity, &mut Transform)>,
    children_q: Query<(&mut MapIcon, &ChildOf)>,
) {
    for (vel, mut transform) in &mut vel_transform_query {
        transform.translation += vel.linvel.extend(0.);
    }
    for (mut map_icon, child_of) in children_q {
        if let Ok((vel, _)) = &vel_transform_query.get(child_of.0) {
            map_icon.update_pos(vel.linvel.as_dvec2());
        }
    }
}

impl Velocity {
    pub const ZERO: Self = Self { linvel: Vec2::ZERO };

    pub fn add_from_endpoint(
        &mut self,
        start_point: DVec2,
        endpoint: DVec2,
        max_speed: f32,
        acceleration: f32,
        time: &Res<Time>,
    ) {
        let direction = (endpoint - start_point).normalize().as_vec2();
        self.linvel += direction * acceleration * time.delta_secs() as f32;

        if self.linvel.length() > max_speed {
            self.linvel = self.linvel.normalize() * max_speed;
        }
    }

    pub fn brake(&mut self, acceleration: f32, time: &Res<Time>) {
        let speed = self.linvel.length();
        let new_speed = (speed - 4.2 * acceleration * time.delta_secs()).max(0.);

        if speed > 0. {
            self.linvel = self.linvel.normalize() * new_speed;
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
        if map_camera.zoom > 0.4 && map_camera.zoom < 10. {
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
pub fn map_to_screen(
    // from map (simulation) to screen (visualisation)
    map_pos: DVec2,
    camera: &MapCamera,
) -> Vec2 {
    let mut pos = map_pos.clone();

    pos -= camera.center;

    pos.x *= camera.zoom;
    pos.y *= -camera.zoom;

    pos.x += camera.viewport_size.x as f64 / 2.;
    pos.y += camera.viewport_size.y as f64 / 2.;

    Vec2 {
        x: pos.x as f32,
        y: pos.y as f32,
    }
}

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

pub struct MapLoD {
    pub show_invidual_ships: bool,
    pub show_fleets: bool,
    pub show_station_labels: bool,
    pub show_ship_labels: bool,
    pub show_routes: bool,
}

pub fn calculate_lod(camera: Res<MapCamera>) -> MapLoD {
    let mut show_invidual_ships = false;
    let mut show_fleets = false;
    let mut show_station_labels = false;
    let mut show_ship_labels = false;
    let show_routes = true; // This is ALWAYS true.

    if camera.zoom > 0. {
        if camera.zoom < 1. {
            show_invidual_ships = true;
            show_station_labels = true;
            show_ship_labels = true;
        } else if camera.zoom < 5. {
            show_fleets = true;
            show_station_labels = true;
        } else {
            show_fleets = true;
        }
    }
    MapLoD {
        show_invidual_ships,
        show_fleets,
        show_station_labels,
        show_ship_labels,
        show_routes,
    }
}

// ############################################################################ Check for visible objects

#[derive(Resource)]
pub struct VisibleMapObjects {
    pub objects: Vec<VisibleMapObject>,
}

pub fn get_visible_map_objects(
    mut commands: Commands,
    current_state: Res<State<MapState>>,
    entities: Query<
        (
            Entity,
            &EntityPosInASector,
            &IconType,
            &Factions,
            &RelationType,
        ),
        With<MapVisible>,
    >,
    camera: Res<MapCamera>,
) {
    let mut visible_objects: Vec<VisibleMapObject> = Vec::new();
    match *current_state.get() {
        MapState::Universe => {}
        MapState::Sector(sector_id) => {
            for (entity, pos, obj_type, faction, relationship) in entities {
                if pos.sector == sector_id {
                    let screen_pos = map_to_screen(pos.pos, &camera);
                    visible_objects.push(VisibleMapObject {
                        entity,
                        map_pos: pos.pos,
                        screen_pos,
                        obj_type: obj_type.clone(),
                        faction: *faction,
                        sector_id,
                        relationship: relationship.clone(),
                    });
                }
            }
        }
    }
    commands.remove_resource::<VisibleMapObjects>();
    commands.insert_resource(VisibleMapObjects {
        objects: visible_objects,
    });
}

// ##################################################### # # # RENDERING # # # ######################################################

// Background Grid (background itself is basically a fixed color)
pub fn render_map_grid(mut commands: Commands) {
    commands.spawn((
        InfiniteGrid,
        InfiniteGridSettings {
            x_axis_color: Color::srgb(0.77, 0.77, 0.77),
            z_axis_color: Color::srgb(0.77, 0.77, 0.77),
            minor_line_color: Color::srgb(0.33, 0.33, 0.33),
            major_line_color: Color::srgb(0.55, 0.55, 0.55),
            fadeout_distance: 10.,     // this is the thing to experiment with
            dot_fadeout_strength: 10., // ---||---
            scale: 5.,
        },
    ));
}

// Objects (map icons)
pub fn render_map_icons(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut visible_objects: Query<&mut VisibleMapObjects>,
) {
    if let Some(mut objects) = visible_objects.iter_mut().next() {
        for object in objects.objects.iter_mut() {
            object.draw(
                &mut commands,
                &asset_server,
                object.faction,
                object.sector_id,
                object.relationship.clone(),
            );
        }
    }
}

// ############## Routes

#[derive(Component, Clone, Debug, Default)]
pub enum Order {
    #[default]
    None,
    Fly,
    Attack,
    Trade,
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

// ##################################################### # # # ORDERS # # # ######################################################

//temp
#[derive(Resource)]
pub struct Unpaused;
pub fn toggle_pause(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    is_unpaused: Option<Res<Unpaused>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        if let Some(_) = is_unpaused {
            commands.remove_resource::<Unpaused>();
        } else {
            commands.insert_resource(Unpaused);
        }
    }
}
//temp

// making orders!!1!!
pub fn order(
    mut commands: Commands,
    selected_q: Query<(Entity, &MapIcon, &ChildOf), With<MapIconSelected>>,
    cursor: Res<MapCursor>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut vel_q: Query<&mut Velocity>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        for (entity, icon, child_of) in selected_q {
            let pos = icon.get_pos();

            if let Ok(mut vel) = vel_q.get_mut(child_of.0) {
                vel.linvel = Vec2::ZERO;
            }
            
            commands
                .entity(entity)
                .insert((MapRoute::new(pos, cursor.pos), Order::Fly));
        }
    }
}

// Fulfilling orders
pub fn fulfill_orders(
    mut commands: Commands,
    orders: Query<(Entity, &Order, &MapRoute, &MapIcon, &ChildOf)>,
    mut map_icons: Query<&mut Velocity>,
    mut transform_q: Query<&mut Transform, Without<Rotated>>,
    time: Res<Time>,
    _unpaused: If<Res<Unpaused>>,
) {
    for (entity, _order, route, icon, child_of) in orders {
        // This also checks if route isn't empty.
        if let Some(endpoint) = route.path_endpoints.last() {
            let parent = child_of.parent();
            let pos = icon.get_pos();
            let acceleration = icon.get_acceleration();

            if let Ok(mut transform) = transform_q.get_mut(parent) {
                let direction = endpoint - pos;

                let target_angle_quat =
                    Quat::from_rotation_z((direction.y.atan2(direction.x) - FRAC_PI_2) as f32);

                let (_, _, target_angle) = target_angle_quat.to_euler(EulerRot::XYZ);
                let (_, _, current_angle) = transform.rotation.to_euler(EulerRot::XYZ);
                let (_, _, default_angle) =
                    Quat::from_rotation_z(2. * time.delta_secs()).to_euler(EulerRot::XYZ);

                let angle_diff = (target_angle - current_angle + PI).rem_euclid(TAU) - PI;

                if angle_diff > 0. {
                    if !(angle_diff > -0.1 && angle_diff < 0.1) {
                        transform.rotate_z(default_angle);
                    } else {
                        transform.rotation = target_angle_quat;
                    }
                } else if angle_diff < 0. {
                    if !(angle_diff > -0.1 && angle_diff < 0.1) {
                        transform.rotate_z(-default_angle);
                    } else {
                        transform.rotation = target_angle_quat;
                    }
                } else {
                    commands.entity(parent).insert(Rotated);
                }
                continue;
            }
            if let Ok(mut vel) = map_icons.get_mut(parent) {
                let max_vel = icon.get_max_vel();
                let distance_to_finish = (endpoint - pos).length();

                // and the fun part!
                let current_speed = vel.linvel.length();
                if distance_to_finish < 1. {
                    vel.linvel = (endpoint - pos).normalize().as_vec2() / 33.;
                    commands.entity(parent).remove::<Rotated>();
                    commands.entity(entity).remove::<MapRoute>();
                    commands.entity(entity).remove::<Order>();
                } else if distance_to_finish <= current_speed as f64 * 42. {
                    vel.brake(acceleration, &time);
                } else {
                    vel.add_from_endpoint(pos, *endpoint, max_vel, acceleration, &time);
                }
            }
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

pub fn select(
    mut commands: Commands,
    selected_q: Query<(Entity, &MapIcon), Without<MapIconSelected>>,
    cursor: Res<MapCursor>,
    mouse: Res<ButtonInput<MouseButton>>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        println!("### ### {}", cursor.pos);
        for (entity, icon) in selected_q {
            let icon_pos = icon.get_pos();
            println!("### ### ### #### {icon_pos}");
            if check_if_clicked_inside_an_object(icon_pos, cursor.pos, 20., 20.) {
                println!("SELECTED A SHIP.");
                commands.entity(entity).insert(MapIconSelected);
            }
        }
    }
}
