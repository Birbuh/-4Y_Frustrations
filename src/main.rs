// extern crate bevy;
mod map;
mod products;
mod ships;

use bevy::{math::DVec2, prelude::*};

use crate::{
    map::{EntityPosInASector, Factions, IconType, MapCamera, MapPlugin, MapState, MapVisible, get_visible_map_objects, render_map_icons}, ships::{Fighter, ShipType},
};

pub fn test_map(mut commands: Commands/*, mut next_state: ResMut<NextState<MapState>>*/) {
    let owner = Factions::Petakians;
    let pos = DVec2::new(13., 17.);
    commands.spawn((
        IconType::Ship(ShipType::Fighter(Fighter::new(
            1,
            DVec2::new(10., 20.),
            owner,
        ))),
        owner,
        EntityPosInASector { sector: 1, pos },
        MapVisible,
    ));

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
        .add_systems(Startup, (test_map, get_visible_map_objects, render_map_icons).chain())
        .add_plugins((DefaultPlugins, MapPlugin))
        .insert_state(MapState::Sector(1))
        .run();
}
