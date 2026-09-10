// extern crate bevy;
mod map;
mod products;
mod ships;

use bevy::{
    math::{DVec2},
    prelude::*,
    window::WindowMode::BorderlessFullscreen,
};

use crate::{
    map::{
        EntityPosInASector, Factions, IconType, MapCamera, MapCursor, MapIcon, MapIconSelected,
        MapPlugin, MapRoute, MapState, MapVisible, Order, RelationType, Velocity, draw_sector,
        fulfill_orders, get_visible_map_objects, order, render_map_icons, render_routes, select,
        spawn_sector, toggle_pause, update_cursor_pos, update_pos_from_velocity, update_ship_pos,
    },
    ships::{Fighter, Ship, ShipType},
};

pub fn test_map(mut commands: Commands /*, mut next_state: ResMut<NextState<MapState>>*/) {
    let owner = Factions::TestEnemy;
    let pos = DVec2::new(13., 17.);
    let icon_type = IconType::Ship(ShipType::Fighter(Fighter::new(
        1,
        pos.clone(),
        owner,
        100.,
        30.,
    )));
    commands
        .spawn((
            icon_type.clone(),
            owner,
            EntityPosInASector { sector: 1, pos },
            MapVisible,
            Transform::from_translation(pos.clone().as_vec2().extend(0.)),
            Velocity::ZERO,
            RelationType::Enemy,
        ))
        .with_children(|parent| {
            parent.spawn((
                MapIcon::new(icon_type.clone(), map::RelationType::Enemy),
                MapRoute::new(pos.clone(), DVec2::new(2070., -2800.)),
                Order::Fly,
            ));
        });

    let owner = Factions::TestAlly;
    let pos = DVec2::new(-333., 37.);
    let icon_type = IconType::Ship(ShipType::Fighter(Fighter::new(
        1,
        pos.clone(),
        owner,
        150.,
        33.,
    )));
    commands
        .spawn((
            icon_type.clone(),
            owner,
            EntityPosInASector { sector: 1, pos },
            MapVisible,
            Transform::from_translation(pos.clone().as_vec2().extend(0.)),
            Velocity::ZERO,
            RelationType::Ally,
        ))
        .with_children(|parent| {
            parent.spawn((
                MapIcon::new(icon_type.clone(), map::RelationType::Player),
                MapRoute::new(pos.clone(), DVec2::new(4370., 200.)),
                Order::Fly,
            ));
        });

    let owner = Factions::Player;
    let pos = DVec2::new(-333., -233.);
    let icon_type = IconType::Ship(ShipType::Fighter(Fighter::new(
        1,
        pos.clone(),
        owner,
        150.,
        33.,
    )));
    commands
        .spawn((
            icon_type.clone(),
            owner,
            EntityPosInASector { sector: 1, pos },
            MapVisible,
            Transform::from_translation(pos.clone().as_vec2().extend(0.)),
            Velocity::ZERO,
            RelationType::Player,
        ))
        .with_children(|parent| {
            parent.spawn((MapIcon::new(icon_type.clone(), RelationType::Player),));
        });
    commands.spawn(Camera2d);
}

fn main() {
    let mut app = App::new();
    app.insert_resource(MapCamera {
        center: DVec2::new(0., 0.),
        zoom: 2.,
        viewport_size: Vec2::new(1920., 1080.),
    })
    .insert_resource(MapCursor { pos: DVec2::ZERO })
    .add_systems(
        Startup,
        (
            test_map,
            spawn_sector,
            get_visible_map_objects,
            render_map_icons,
        )
            .chain(),
    )
    .add_systems(
        FixedUpdate,
        (
            update_cursor_pos,
            toggle_pause,
            update_pos_from_velocity,
            update_ship_pos,
            fulfill_orders,
        )
            .chain(),
    )
    .add_systems(
        Update, 
        (
            select, 
            order, 
            render_routes, 
            draw_sector
        )
    )
    .add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                mode: BorderlessFullscreen(MonitorSelection::Primary),
                title: "-4Y: Frustration".to_string(),
                ..default()
            }),
            close_when_requested: true,
            ..default()
        }),
        MapPlugin,
    ))
    .insert_state(MapState::Sector(1))
    .run();
}
