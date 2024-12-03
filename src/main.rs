#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::time::Duration;

use bevy::{color::palettes::css::RED, prelude::*};

mod components;
mod resources;
mod systems;

use resources::*;
use systems::*;

const WINDOW_WIDTH: f32 = 700.0;
const WINDOW_HEIGHT: f32 = 400.0;

const BIRB_HEIGHT: f32 = 24.0;
const BIRB_WIDTH: f32 = 34.0;

const PIPE_HEIGHT: f32 = 320.0;
const PIPE_WIDTH: f32 = 52.0;

const DEFAULT_GRAVITY: f32 = 200.0;

const DRAW_COLLIDERS: bool = false;

const SCORE_UPDATE_INTERVAL_MS: u64 = 750;

const PIPE_SPAWN_INTERVAL_MIN_MS: u64 = 1500;
const PIPE_SPAWN_INTERVAL_MAX_MS: u64 = 3000;

const MIN_VELOCITY: f32 = -250.0;
const MAX_VELOCITY: f32 = 250.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Flappy Birb".to_string(),
                name: Some("flappy_birb".to_string()),
                resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                resizable: false,
                enabled_buttons: bevy::window::EnabledButtons {
                    minimize: true,
                    maximize: false,
                    close: true,
                },
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .init_state::<GameState>()
        .insert_resource(Gravity(DEFAULT_GRAVITY))
        .insert_resource(PipeSpawnTimer(Timer::new(
            Duration::from_millis(PIPE_SPAWN_INTERVAL_MIN_MS),
            TimerMode::Repeating,
        )))
        .insert_resource(ScoreTimer(Timer::new(
            Duration::from_millis(SCORE_UPDATE_INTERVAL_MS),
            TimerMode::Repeating,
        )))
        .insert_resource(Score(0))
        .insert_resource(DrawColliders(DRAW_COLLIDERS))
        .add_systems(
            Update,
            (score_tick, on_space_pressed, apply_gravity, apply_velocity)
                .chain()
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            (spawn_pipe, move_pipes, check_birb_collisions)
                .chain()
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(Update, await_restart.run_if(in_state(GameState::Loss)))
        .add_systems(
            Update,
            draw_collider_gizmos.run_if(|draw_colliders: Res<DrawColliders>| **draw_colliders),
        )
        .add_systems(OnEnter(GameState::Loss), on_loss)
        .add_systems(OnExit(GameState::Loss), on_restart)
        .add_systems(
            PostUpdate,
            on_score_change.run_if(resource_changed::<Score>),
        )
        .run();
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameState {
    #[default]
    Playing,
    Loss,
}
