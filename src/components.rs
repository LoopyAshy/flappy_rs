use bevy::prelude::*;

#[derive(Debug, Component)]
/// A marker component for the score text
pub struct ScoreText;

#[derive(Debug, Component, Deref, DerefMut)]
/// A component for the y velocity of an entity as we do not care about x movement in this game, this component allows the entity to move each frame based on the stored velocity
pub struct Velocity(pub f32);

#[derive(Debug, Component, Deref, DerefMut)]
/// A component for a rectangle collider for a entity
pub struct Collider(pub Rect);

#[derive(Debug, Component)]
/// A marker component for the birb
pub struct Birb;

#[derive(Debug, Component)]
/// A marker component for pipes
pub struct Pipe;

#[derive(Debug, Component)]
/// A marker component for the loss text
pub struct LossText;
