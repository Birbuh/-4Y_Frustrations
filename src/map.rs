use std::default;

use bevy::{
    camera::Camera2d,
    dev_tools::infinite_grid::{InfiniteGrid, InfiniteGridPlugin, InfiniteGridSettings},
    ecs::resource::Resource,
    input::{
        ButtonInput,
        keyboard::{Key::ColorF2Yellow, KeyCode},
        mouse::{AccumulatedMouseMotion, MouseWheel},
    },
    math::DVec2,
    mesh::PrimitiveTopology::{LineList, LineStrip},
    prelude::*,
    reflect::tuple_struct::TupleStructFieldIter,
    ui::Selected,
    window::PrimaryWindow,
};

use crate::{
    ships::{Fighter, ShipType, Ship},
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

#[derive(Component, Clone, Copy, Debug)]
pub enum Factions {
    Petakians,
    Furgians,
}

impl Factions {
    pub fn get_color(&self) -> Color {
        match self {
            Self::Furgians => Color::srgb(0.5, 0.33, 0.01),
            Self::Petakians => Color::srgb(0.77, 0.33, 0.67)
        }
    }
}

#[derive(Component, Clone, Debug)]
pub enum IconType {
    Ship(ShipType),
}

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

#[derive(Component)]
pub struct MapIcon {
    // pub size: f32,
    pub icon_type: IconType,
    pub relations_type: RelationType,
}

impl MapIcon {
    pub fn new(icon_type: IconType, relations_type: RelationType) -> Self {
        Self {
            icon_type,
            relations_type
        }
    }
    pub fn get_pos(&self) -> DVec2 {
        match &self.icon_type {
            IconType::Ship(ship) => match ship {
                ShipType::Fighter(fighter) => fighter.pos,
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
}

impl VisibleMapObject {
    pub fn draw(
        &mut self,
        commands: &mut Commands,
        asset_server: &AssetServer,
        faction: Factions,
        sector_id: u16,
    ) {
        let (image, obj_type): (Handle<Image>, IconType) = match &self.obj_type {
            IconType::Ship(ship_type) => match ship_type {
                ShipType::Fighter(_) => (
                    asset_server.load("icons/fighter.png"),
                    IconType::Ship(ShipType::Fighter(Fighter::new(
                        sector_id,
                        self.map_pos,
                        faction,
                    ))),
                ),
            },
        };
        let entity_commands = commands.spawn((
            Sprite { image, ..default() },
            Transform::from_translation(self.map_pos.as_vec2().extend(0.)),
            EntityPosInASector {
                sector: sector_id,
                pos: self.map_pos,
            },
            match &obj_type {
                IconType::Ship(ship_type) => match ship_type {
                    ShipType::Fighter(fighter) => fighter.clone(),
                },
            },
        ));
        println!("Spawned!");

        self.entity = entity_commands.id();
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
    camera: MapCamera,
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
    entities: Query<(Entity, &EntityPosInASector, &IconType, &Factions), With<MapVisible>>,
    camera: Res<MapCamera>,
) {
    let mut visible_objects: Vec<VisibleMapObject> = Vec::new();
    match *current_state.get() {
        MapState::Universe => {}
        MapState::Sector(sector_id) => {
            for (entity, pos, obj_type, faction) in entities {
                if pos.sector == sector_id {
                    let screen_pos = map_to_screen(pos.pos, &camera);
                    visible_objects.push(VisibleMapObject {
                        entity,
                        map_pos: pos.pos,
                        screen_pos,
                        obj_type: obj_type.clone(),
                        faction: *faction,
                        sector_id,
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

pub fn render_routes(
    mut gizmos: Gizmos,
    route_q: Query<(&MapRoute, &MapIcon), With<Order>>,
) {
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

pub fn spawn_sector(mut commands: Commands) { // It's here, because it's always the same. It spawns when in MapState::Sector(_), the ID does nothing here.
    let sectors = vec![
        ( Sector { center: DVec2::new(0., 0.), radius: 6700. }, Factions::Furgians )
    ];
    for (sector, owner) in sectors {
        commands.spawn((
            sector.clone(),
            owner.clone(),
            Transform::from_translation(sector.center.as_vec2().extend(0.).clone())
        ));
    }
}

pub fn draw_sector(mut gizmos: Gizmos, sector_q: Query<(&Sector, &Factions)>) {
    for (sector, owner) in sector_q {
        gizmos.circle_2d(Isometry2d::from_xy(sector.center.x as f32, sector.center.y as f32), sector.radius as f32, owner.get_color());
    }
}

// ##################################################### # # # ORDERS # # # ######################################################
