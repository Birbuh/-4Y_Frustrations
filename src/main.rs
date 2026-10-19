// extern crate bevy;
mod map;
mod npcs;
mod ships;

use std::collections::HashMap;

use bevy::{math::DVec2, prelude::*, window::WindowMode::BorderlessFullscreen};

use crate::{
    map::{
        MapCamera, MapCursor, MapPlugin, MapState, Unpaused, add_ship_from_input, draw_sector,
        fulfill_orders, get_visible_map_objects, order, render_map_icons, render_routes, select,
        spawn_sector, toggle_pause, update_cursor_pos, update_pos_from_velocity, update_ship_pos,
    },
    npcs::{
        ShipAiCooldownTimer, reset_finished_ai_cooldown_timer, ships_ai, tick_ai_cooldown_timer,
    },
};

fn test_map(mut commands: Commands) {
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
    .insert_resource(ShipAiCooldownTimer(HashMap::new()))
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
            tick_ai_cooldown_timer,
            update_pos_from_velocity,
            update_ship_pos,
            fulfill_orders,
            ships_ai,
            reset_finished_ai_cooldown_timer,
        )
            .chain()
            .run_if(resource_exists::<Unpaused>),
    )
    .add_systems(
        Update,
        (
            update_cursor_pos,
            add_ship_from_input,
            toggle_pause,
            (order, select).chain(),
            render_routes,
            draw_sector,
        ),
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
