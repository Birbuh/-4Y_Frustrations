use std::default;

use bevy::{
    camera::Camera2d,
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

use crate::ships::ShipType;

pub enum Factions {
    Petakians,
    Furgians,
    Papug,
}

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
    Sector1(Entity),
    Sector2,
    Sector3,
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
pub struct SectorPos {
    pub sector: Entity,
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

pub struct VisibleMapObject {
    pub entity: Entity,
    pub map_pos: DVec2,
    pub screen_pos: Vec2,
}

#[derive(Resource)]
pub struct VisibleMapObjects {
    pub objects: Vec<VisibleMapObject>,
}

// ##################################################################### updates the camera.
pub fn update_map_camera(
    // keyboard: Res<ButtonInput<KeyCode>>, // preserved for future
    mouse: Res<ButtonInput<MouseButton>>,
    mut map_camera: ResMut<MapCamera>,
    mut mouse_wheel: MessageReader<MouseWheel>,
    mouse_motion: Res<AccumulatedMouseMotion>,
) {
    for event in mouse_wheel.read() {
        map_camera.zoom += event.y as f64; // update the map zoom by the amount was scrolled.
        // For future development.
        // match event.unit {
        //     bevy::input::mouse::MouseScrollUnit::Line => {},
        //     bevy::input::mouse::MouseScrollUnit::Pixel => {}
        // }
    }

    if mouse.just_pressed(MouseButton::Right) {
        let delta_mouse_motion = mouse_motion.delta; // get mouse delta from the last frame

        map_camera.update_pos_from_vec2(&delta_mouse_motion); // update the map's centre pos
    }
}

// ################################################################### calculates positions
pub fn map_to_screen(
    // from map (simulation) to screen (visualisation)
    map_pos: DVec2,
    camera: MapCamera,
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

// pub fn get_visible_map_objects(
//     current_state: State<MapState>,
//     entities: Query<(Entity, &SectorPos), With<MapVisible>>,
//     camera: Res<MapCamera>
// ) {
//     let mut visible_objects: Vec<VisibleMapObject> = Vec::new();
//     match current_state.get() {
//         MapState::Universe => {}
//         MapState::Sector1(sector_id) => {
//             for (entity, pos) in entities {
//                 if pos.sector == *sector_id {
//                     // let screen_pos = camera.center;
//                     visible_objects.push(VisibleMapObject { entity, map_pos: pos.pos, });
//                 }
//             }
//         }
//     }
// }
