#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::time::Duration;

use bevy::{color::palettes::css::RED, prelude::*};

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
                title: "Flappy Bird".to_string(),
                name: Some("flappy_bird".to_string()),
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
            (
                update_score,
                on_space_pressed,
                apply_gravity,
                apply_velocity,
            )
                .chain()
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            (spawn_pipe, move_pipes, on_birb_collide)
                .chain()
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(Update, await_restart.run_if(in_state(GameState::Loss)))
        .add_systems(
            Update,
            draw_collider_gizmos.run_if(|draw_colliders: Res<DrawColliders>| draw_colliders.0),
        )
        .run();
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum GameState {
    #[default]
    Playing,
    Loss,
}

//Resources

#[derive(Debug, Resource)]
/// A resource for storing the current score
struct Score(u32);

#[derive(Debug, Resource)]
/// A timer for tracking the time between updating the score
struct ScoreTimer(Timer);

#[derive(Debug, Resource)]
/// A resource for drawing the rectangle colliders of entities when true, this is for debugging purposes and is usually false or not inserted
struct DrawColliders(bool);

#[derive(Debug, Resource)]
/// A resource for the gravity of the game, we subtract it from the velocity of the birb every frame
struct Gravity(f32);

#[derive(Debug, Resource)]
/// A timer for tracking the time between spawning pipes
struct PipeSpawnTimer(Timer);

//Components

#[derive(Debug, Component)]
/// A marker component for the score text
struct ScoreText;

#[derive(Debug, Component)]
/// A component for the y velocity of an entity as we do not care about x movement in this game, this component allows the entity to move each frame based on the stored velocity
struct Velocity(f32);

#[derive(Debug, Component)]
/// A component for a rectangle collider for a entity
struct Collider(Rect);

#[derive(Debug, Component)]
/// A marker component for the birb
struct Birb;

#[derive(Debug, Component)]
/// A marker component for pipes
struct Pipe;

//Bundles

#[derive(Debug, Bundle)]
struct BirbBundle {
    birb: Birb,
    velocity: Velocity,
    sprite: SpriteBundle,
    collider: Collider,
}

impl BirbBundle {
    fn new(asset_server: &AssetServer) -> Self {
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
struct PipeBundle {
    pipe: Pipe,
    transform: SpriteBundle,
    collider: Collider,
}

impl PipeBundle {
    fn new(asset_server: &AssetServer, position: Vec2) -> Self {
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
    fn flipped(mut self) -> Self {
        self.transform.transform.rotate_x(std::f32::consts::PI);
        self
    }
}

//Systems

/// Updates the score text to the current score
fn draw_score(score: u32, mut text: Query<&mut Text, With<ScoreText>>) {
    for mut text in text.iter_mut() {
        text.sections[0].value = score.to_string();
    }
}

/// Waits for the player to press enter to restart the game if they lost
fn await_restart(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut birb_query: Query<(&mut Transform, &mut Velocity), With<Birb>>,
    pipes_query: Query<Entity, With<Pipe>>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    mut score: ResMut<Score>,
) {
    if keyboard_input.just_pressed(KeyCode::Enter) {
        for (mut transform, mut velocity) in birb_query.iter_mut() {
            transform.translation.y = 0.0;
            velocity.0 = 0.0;
        }
        for pipe in pipes_query.iter() {
            commands.entity(pipe).despawn_recursive();
        }
        score.0 = 0;
        next_state.set(GameState::Playing);
    }
}

/// Updates the score every 750ms by 1 point
fn update_score(
    time: Res<Time>,
    mut score: ResMut<Score>,
    mut score_timer: ResMut<ScoreTimer>,
    text: Query<&mut Text, With<ScoreText>>,
) {
    if score_timer.0.tick(time.delta()).just_finished() {
        score.0 += 1;
        draw_score(score.0, text);
    }
}

/// Sets up the initial state of the game
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(TextBundle::from_sections([TextSection::new(
            "0",
            TextStyle {
                font_size: 40.0,
                color: Color::WHITE,
                ..default()
            },
        )]))
        .insert(ScoreText);
    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 10.0),
        ..default()
    });
    commands.spawn(BirbBundle::new(&asset_server));
}

/// Checks if the birb has collided with any pipes, if so, sets the game state to loss
fn on_birb_collide(
    birb_query: Query<(&Collider, &Transform), With<Birb>>,
    pipes: Query<(&Collider, &Transform), With<Pipe>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (birb_collider, birb_transform) in birb_query.iter() {
        let birb_rect = Rect {
            min: birb_transform.translation.truncate() + birb_collider.0.min,
            max: birb_transform.translation.truncate() + birb_collider.0.max,
        };
        for (pipe_collider, pipe_transform) in pipes.iter() {
            let pipe_rect = Rect {
                min: pipe_transform.translation.truncate() + pipe_collider.0.min,
                max: pipe_transform.translation.truncate() + pipe_collider.0.max,
            };
            if birb_rect.max.x > pipe_rect.min.x
                && birb_rect.min.x < pipe_rect.max.x
                && birb_rect.max.y > pipe_rect.min.y
                && birb_rect.min.y < pipe_rect.max.y
            {
                next_state.set(GameState::Loss);
            }
        }
    }
}

/// Spawns pipes at a random height and gap between the top and bottom pipes
fn spawn_pipe(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut timer: ResMut<PipeSpawnTimer>,
    time: Res<Time>,
    score: Res<Score>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        // Updates the delay between the pipes based on the score to make the game harder as the player progresses
        timer.0.set_duration(
            (Duration::from_millis(PIPE_SPAWN_INTERVAL_MAX_MS)
                - Duration::from_millis(score.0 as u64 / 2))
            .min(Duration::from_millis(PIPE_SPAWN_INTERVAL_MIN_MS)),
        );
        const HALF_HEIGHT: f32 = WINDOW_HEIGHT / 2.0;
        const HALF_WIDTH: f32 = WINDOW_WIDTH / 2.0;
        const HEIGHT_DIFF: i32 = 75;
        let y = fastrand::i32(-HALF_HEIGHT as i32 - HEIGHT_DIFF..-HALF_HEIGHT as i32 + HEIGHT_DIFF);
        const MAX_GAP_DIFF: i32 = 95;
        const MIN_GAP: i32 = 400;
        let gap = fastrand::i32(MIN_GAP..(MIN_GAP + MAX_GAP_DIFF));
        commands.spawn(PipeBundle::new(
            &asset_server,
            Vec2::new(HALF_WIDTH + 100.0, y as f32),
        ));
        commands.spawn(
            PipeBundle::new(
                &asset_server,
                Vec2::new(HALF_WIDTH + 100.0, y as f32 + gap as f32),
            )
            .flipped(),
        );
    }
}

/// Draws a rectangle gizmo for all entities with a Collider and Transform component
fn draw_collider_gizmos(query: Query<(&Collider, &Transform)>, mut gizmos: Gizmos) {
    for (collider, transform) in query.iter() {
        gizmos.rect_2d(
            transform.translation.truncate(),
            0.0,
            collider.0.size(),
            RED,
        );
    }
}

/// Moves all pipes to the left and despawns them when they are off-screen
fn move_pipes(
    mut query: Query<(Entity, &mut Transform), With<Pipe>>,
    time: Res<Time>,
    mut commands: Commands,
) {
    const PIPE_SPEED: f32 = 100.0;
    for (entity, mut transform) in query.iter_mut() {
        transform.translation.x -= PIPE_SPEED * time.delta_seconds();
        // Despawns pipes when they are off-screen by 50 units
        if transform.translation.x < -WINDOW_WIDTH / 2.0 - 50.0 {
            commands.entity(entity).despawn_recursive();
            println!("despawned pipe");
        }
    }
}

/// On space pressed, sets the velocity of all entities with a Velocity component to the maximum upward velocity to simulate flapping wings and gaining altitude
fn on_space_pressed(mut query: Query<&mut Velocity>, keyboard_input: Res<ButtonInput<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        for mut velocity in query.iter_mut() {
            velocity.0 = 150.0;
        }
    }
}

/// Applies gravity to all entities with a Velocity component
fn apply_gravity(mut query: Query<&mut Velocity>, gravity: Res<Gravity>, time: Res<Time>) {
    for mut velocity in query.iter_mut() {
        velocity.0 -= gravity.0 * time.delta_seconds();
        velocity.0 = velocity.0.clamp(MIN_VELOCITY, MAX_VELOCITY);
    }
}

/// Applies velocity to all entities with a Transform and Velocity component
fn apply_velocity(mut query: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    for (mut transform, velocity) in query.iter_mut() {
        if velocity.0 >= 0.0 && transform.translation.y >= ((WINDOW_HEIGHT - BIRB_HEIGHT) / 2.0)
            || velocity.0 <= 0.0
                && transform.translation.y <= -((WINDOW_HEIGHT - BIRB_HEIGHT) / 2.0)
        {
            continue;
        }
        transform.translation += Vec3::new(0.0, velocity.0, 0.0) * time.delta_seconds();
    }
}
