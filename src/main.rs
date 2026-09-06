// extern crate bevy;
mod map;
mod products;
mod ships;

use bevy::{math::DVec2, prelude::*};

use crate::{
    map::{EntityPosInASector, Factions, IconType, MapCamera, MapIcon, MapPlugin, MapRoute, MapState, MapVisible, Order, draw_sector, get_visible_map_objects, render_map_icons, render_routes, spawn_sector}, ships::{Fighter, Ship, ShipType},
};

pub fn test_map(mut commands: Commands/*, mut next_state: ResMut<NextState<MapState>>*/) {
    let owner = Factions::Petakians;
    let pos = DVec2::new(13., 17.);
    let icon_type = IconType::Ship(ShipType::Fighter(Fighter::new(
                1,
                pos.clone(),
                owner,
            )));
    commands.spawn((
        icon_type.clone(),
        owner,
        EntityPosInASector { sector: 1, pos },
        MapVisible,
    )).with_children(|parent| {
        parent.spawn((
            MapIcon::new(icon_type.clone(), map::RelationType::Friendly),
            MapRoute::new(pos.clone(), DVec2::new(1000., 100.)),
            Order::Fly,
        ));
    });

    commands.spawn(Camera2d);
    // next_state.set(MapState::Sector(1));
}

fn main() {
    let mut app = App::new();
    app.insert_resource(MapCamera {
            center: DVec2::new(0., 0.),
            zoom: 2.,
            viewport_size: Vec2::new(160., 90.)
        })
        .add_systems(Startup, (test_map, spawn_sector, get_visible_map_objects, render_map_icons).chain())
        .add_systems(Update, (render_routes, draw_sector))
        .add_plugins((DefaultPlugins, MapPlugin))
        .insert_state(MapState::Sector(1))
        .run();
}
