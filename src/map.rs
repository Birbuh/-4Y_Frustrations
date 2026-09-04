use std::default;

use bevy::{
    camera::Camera2d,
    dev_tools::infinite_grid::{InfiniteGrid, InfiniteGridPlugin, InfiniteGridSettings},
    ecs::resource::Resource,
    input::{
        ButtonInput,
        keyboard::KeyCode,
        mouse::{AccumulatedMouseMotion, MouseWheel},
    },
    math::DVec2,
    prelude::*,
    reflect::tuple_struct::TupleStructFieldIter,
    ui::Selected,
    window::PrimaryWindow,
};

use crate::{
    map::{self, IconType::Ship}, ships::{Fighter, ShipType},
};

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        // app.add_plugins(InfiniteGridPlugin);
        app.add_systems(FixedUpdate, (update_map_camera, update_camera_from_map_camera).chain());
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub enum Factions {
    Petakians,
    Furgians,
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
    pub size: f32,
    pub icon_type: IconType,
    pub relations_type: RelationType,
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

pub struct MapLoD {
    pub show_invidual_ships: bool,
    pub show_fleets: bool,
    pub show_station_labels: bool,
    pub show_ship_labels: bool,
    pub show_routes: bool,
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
                    asset_server.load("fighter.png"),
                    IconType::Ship(ShipType::Fighter(Fighter::new(
                        sector_id,
                        self.map_pos,
                        faction,
                    ))),
                ),
            },
        };
        let entity_commands = commands.spawn((
            Sprite {
                image,
                ..default()
            },
            Transform::from_translation(self.screen_pos.extend(0.)),
            EntityPosInASector {
                sector: sector_id,
                pos: self.map_pos,
            },
            match &obj_type {
                Ship(ship_type) => match ship_type {
                    ShipType::Fighter(fighter) => fighter.clone()
                }
            }
        ));

        self.entity = entity_commands.id();
    }
}

#[derive(Resource)]
pub struct VisibleMapObjects {
    pub objects: Vec<VisibleMapObject>,
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
        if map_camera.zoom > 0. && map_camera.zoom < 10. {
            map_camera.zoom += event.y as f64 / 5.; // update the map zoom by the amount was scrolled.
        } else if map_camera.zoom < 0.{
            if event.y >= 0. {
                map_camera.zoom += event.y as f64 / 5.; // update the map zoom by the amount was scrolled IF IT'S ZOOMING IN.
            }
        } else {
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
    commands.insert_resource(VisibleMapObjects { objects: visible_objects });
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
pub fn render_map_icons(mut commands: Commands, asset_server: Res<AssetServer>, mut visible_objects: Query<&mut VisibleMapObjects>) {
    if let Some(mut objects) = visible_objects.iter_mut().next() {
        for object in objects.objects.iter_mut() {
            object.draw(&mut commands, &asset_server, object.faction, object.sector_id);
        }
    }
}
