use crate::bundles::*;
use crate::components::*;
use crate::resources::*;
use crate::*;
use bevy::prelude::*;

/// Spawns the loss text when the player loses
pub fn on_loss(mut commands: Commands) {
    let text = TextBundle::from_section(
        "You lost!\nPress Enter to Restart",
        TextStyle {
            font_size: 40.0,
            color: Color::WHITE,
            ..default()
        },
    )
    .with_text_justify(JustifyText::Center)
    .with_style(Style {
        position_type: PositionType::Absolute,
        margin: UiRect::all(Val::Auto),
        display: Display::Block,
        ..default()
    });

    commands.spawn((text, LossText));
}

/// Resets the game state to default values
pub fn on_restart(
    mut commands: Commands,
    loss_text: Query<Entity, With<LossText>>,
    mut score: ResMut<Score>,
    mut score_text: Query<&mut Text, With<ScoreText>>,
    mut birb: Query<(&mut Transform, &mut Velocity), With<Birb>>,
    pipes: Query<Entity, With<Pipe>>,
) {
    for loss_text in loss_text.iter() {
        commands.entity(loss_text).despawn_recursive();
    }
    **score = 0;
    for mut score_text in score_text.iter_mut() {
        score_text.sections[0].value = score.to_string();
    }
    for (mut transform, mut velocity) in birb.iter_mut() {
        transform.translation.y = 0.0;
        **velocity = 0.0;
    }
    for pipe in pipes.iter() {
        commands.entity(pipe).despawn_recursive();
    }
}

/// Waits for the player to press enter to restart the game if they lost
pub fn await_restart(
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
            **velocity = 0.0;
        }
        for pipe in pipes_query.iter() {
            commands.entity(pipe).despawn_recursive();
        }
        **score = 0;
        next_state.set(GameState::Playing);
    }
}

/// Updates the score every 750ms by 1 point
pub fn score_tick(time: Res<Time>, mut score: ResMut<Score>, mut score_timer: ResMut<ScoreTimer>) {
    if score_timer.tick(time.delta()).just_finished() {
        **score += 1;
    }
}

/// Updates the score text to the current score
pub fn on_score_change(score: Res<Score>, mut text: Query<&mut Text, With<ScoreText>>) {
    for mut text in text.iter_mut() {
        text.sections[0].value = score.to_string();
    }
}

/// Sets up the initial state of the game
pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
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
pub fn on_birb_collide(
    birb_query: Query<(&Collider, &Transform), With<Birb>>,
    pipes: Query<(&Collider, &Transform), With<Pipe>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (birb_collider, birb_transform) in birb_query.iter() {
        let birb_rect = Rect {
            min: birb_transform.translation.truncate() + birb_collider.min,
            max: birb_transform.translation.truncate() + birb_collider.max,
        };
        for (pipe_collider, pipe_transform) in pipes.iter() {
            let pipe_rect = Rect {
                min: pipe_transform.translation.truncate() + pipe_collider.min,
                max: pipe_transform.translation.truncate() + pipe_collider.max,
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
pub fn spawn_pipe(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut timer: ResMut<PipeSpawnTimer>,
    time: Res<Time>,
    score: Res<Score>,
) {
    if timer.tick(time.delta()).just_finished() {
        // Updates the delay between the pipes based on the score to make the game harder as the player progresses
        timer.set_duration(
            (Duration::from_millis(PIPE_SPAWN_INTERVAL_MAX_MS)
                - Duration::from_millis(**score as u64 / 2))
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
pub fn draw_collider_gizmos(query: Query<(&Collider, &Transform)>, mut gizmos: Gizmos) {
    for (collider, transform) in query.iter() {
        gizmos.rect_2d(transform.translation.truncate(), 0.0, collider.size(), RED);
    }
}

/// Moves all pipes to the left and despawns them when they are off-screen
pub fn move_pipes(
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
pub fn on_space_pressed(
    mut query: Query<&mut Velocity>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        for mut velocity in query.iter_mut() {
            **velocity = 150.0;
        }
    }
}

/// Applies gravity to all entities with a Velocity component
pub fn apply_gravity(mut query: Query<&mut Velocity>, gravity: Res<Gravity>, time: Res<Time>) {
    for mut velocity in query.iter_mut() {
        **velocity -= **gravity * time.delta_seconds();
        **velocity = velocity.clamp(MIN_VELOCITY, MAX_VELOCITY);
    }
}

/// Applies velocity to all entities with a Transform and Velocity component
pub fn apply_velocity(mut query: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    for (mut transform, velocity) in query.iter_mut() {
        if **velocity >= 0.0 && transform.translation.y >= ((WINDOW_HEIGHT - BIRB_HEIGHT) / 2.0)
            || **velocity <= 0.0
                && transform.translation.y <= -((WINDOW_HEIGHT - BIRB_HEIGHT) / 2.0)
        {
            continue;
        }
        transform.translation += Vec3::new(0.0, **velocity, 0.0) * time.delta_seconds();
    }
}
