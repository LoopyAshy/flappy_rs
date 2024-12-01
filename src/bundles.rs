use crate::components::*;
use crate::*;
use bevy::prelude::*;

#[derive(Debug, Bundle)]
pub struct BirbBundle {
    birb: Birb,
    velocity: Velocity,
    sprite: SpriteBundle,
    collider: Collider,
}

impl BirbBundle {
    pub fn new(asset_server: &AssetServer) -> Self {
        Self {
            birb: Birb,
            sprite: SpriteBundle {
                texture: asset_server.load("bluebird-midflap.png"),
                ..default()
            },
            velocity: Velocity(0.0),
            collider: Collider(Rect {
                min: Vec2::new(-BIRB_WIDTH / 2.0, -BIRB_HEIGHT / 2.0),
                max: Vec2::new(BIRB_WIDTH / 2.0, BIRB_HEIGHT / 2.0),
            }),
        }
    }
}

#[derive(Debug, Bundle)]
pub struct PipeBundle {
    pipe: Pipe,
    transform: SpriteBundle,
    collider: Collider,
}

impl PipeBundle {
    pub fn new(asset_server: &AssetServer, position: Vec2) -> Self {
        Self {
            pipe: Pipe,
            transform: SpriteBundle {
                texture: asset_server.load("pipe-green.png"),
                transform: Transform::from_translation(position.extend(1.0)),
                ..default()
            },
            collider: Collider(Rect {
                min: Vec2::new(-PIPE_WIDTH / 2.0, -PIPE_HEIGHT / 2.0),
                max: Vec2::new(PIPE_WIDTH / 2.0, PIPE_HEIGHT / 2.0),
            }),
        }
    }

    /// Flips the pipe upside down
    pub fn flipped(mut self) -> Self {
        self.transform.transform.rotate_x(std::f32::consts::PI);
        self
    }
}
