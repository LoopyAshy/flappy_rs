use bevy::prelude::*;
use crate::*;

#[derive(Debug, Default, Component, Clone)]
#[require(Text(|| Text::new("0")))]
/// A marker component for the score text
pub struct ScoreText;

#[derive(Debug, Default, Component, Deref, DerefMut, Clone)]
#[require(Transform)]
/// A component for the y velocity of an entity as we do not care about x movement in this game, this component allows the entity to move each frame based on the stored velocity
pub struct Velocity(pub f32);

#[derive(Debug, Default, Component, Deref, DerefMut, Clone)]
#[require(Transform)]
/// A component for a rectangle collider for a entity
pub struct Collider(pub Rect);

#[derive(Debug, Default, Component, Clone)]
#[require(Velocity, Collider, Sprite)]
/// A marker component for the birb
pub struct Birb;

impl Birb {
    pub fn new(asset_server: &AssetServer) -> (Self, Collider, Sprite) {
        (Self, Collider(Rect {
            min: Vec2::new(-BIRB_WIDTH / 2.0, -BIRB_HEIGHT / 2.0),
            max: Vec2::new(BIRB_WIDTH / 2.0, BIRB_HEIGHT / 2.0),
        }), Sprite::from_image(asset_server.load::<Image>("bluebird-midflap.png")))
    }
}

#[derive(Debug, Default, Component, Clone)]
#[require(Collider, Sprite)]
/// A marker component for pipes
pub struct Pipe;

impl Pipe {
    pub fn new(asset_server: &AssetServer) -> (Self, Sprite, Collider) {
        (Self, Sprite::from_image(asset_server.load::<Image>("pipe-green.png")), Collider(Rect {
            min: Vec2::new(-PIPE_WIDTH / 2.0, -PIPE_HEIGHT / 2.0),
            max: Vec2::new(PIPE_WIDTH / 2.0, PIPE_HEIGHT / 2.0),
        }))
    }
}

#[derive(Debug, Default, Component, Clone)]
/// A marker component for the loss text
pub struct LossText;
