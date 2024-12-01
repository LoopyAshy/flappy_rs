use bevy::prelude::*;

#[derive(Debug, Resource, Deref, DerefMut)]
/// A resource for storing the current score
pub struct Score(pub u32);

#[derive(Debug, Resource, Deref, DerefMut)]
/// A timer for tracking the time between updating the score
pub struct ScoreTimer(pub Timer);

#[derive(Debug, Resource, Deref, DerefMut)]
/// A resource for drawing the rectangle colliders of entities when true, this is for debugging purposes and is usually false or not inserted
pub struct DrawColliders(pub bool);

#[derive(Debug, Resource, Deref, DerefMut)]
/// A resource for the gravity of the game, we subtract it from the velocity of the birb every frame
pub struct Gravity(pub f32);

#[derive(Debug, Resource, Deref, DerefMut)]
/// A timer for tracking the time between spawning pipes
pub struct PipeSpawnTimer(pub Timer);
