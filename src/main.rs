// extern crate bevy;
mod map;
mod npcs;
mod ships;

use std::collections::HashMap;

use bevy::{math::DVec2, prelude::*, window::WindowMode::BorderlessFullscreen};
use bevy_rapier2d::{plugin::{NoUserData, RapierPhysicsPlugin}, render::RapierDebugRenderPlugin};

use crate::{
    map::{
        MapCamera, MapCursor, MapPlugin, MapState, Paused, Unpaused, add_ship_from_input,
        draw_sector, fulfill_orders, order,
        render_routes, /* select, */ spawn_sector, toggle_pause, update_cursor_pos,
        update_pos_from_velocity, update_ship_pos,
    }, npcs::{
        ShipAiCooldownTimer, reset_finished_ai_cooldown_timer, ships_ai, tick_ai_cooldown_timer,
    }, ships::{ShootCooldownTimer, ship_bullet_collision, spawn_ship_health_bars, update_health_bars},
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
    .insert_resource(ShootCooldownTimer(Timer::from_seconds(0.2, TimerMode::Once)))
    .insert_resource(Paused)
    .add_systems(
        Startup,
        (
            test_map,
            spawn_sector,
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
            (add_ship_from_input, spawn_ship_health_bars).chain(),
            toggle_pause,
            update_health_bars,
            (order, /* select */).chain(),
            render_routes,
            draw_sector,
            ship_bullet_collision,
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
        RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0),
    ))
    .insert_state(MapState::Sector(1))
    .run();
}
