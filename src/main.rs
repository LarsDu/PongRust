use bevy::prelude::*;

/* -- CONSTANTS -- */
// SCREEN
const SCREEN_WIDTH: f32 = 800.0;
const SCREEN_HEIGHT: f32 = 640.0;

// PADDLES
const PADDLE_OFFSET: f32 = 80.0;
const LEFT_PADDLE_POS: Vec2 = Vec2::new(-SCREEN_WIDTH / 2.0 + PADDLE_OFFSET, 0.0);
const RIGHT_PADDLE_POS: Vec2 = Vec2::new(SCREEN_WIDTH / 2.0 - PADDLE_OFFSET, 0.0);
const PADDLE_DIMS: Vec2 = Vec2::new(WALL_THICKNESS, 60.0);

// PUCK
const PUCK_DIMS: Vec2 = Vec2::new(WALL_THICKNESS, WALL_THICKNESS);
const PUCK_SPAWN_POS: Vec2 = Vec2::new(0.0, 0.0);
const PUCK_SPEED: f32 = 350.0;
const INITIAL_PUCK_DIRECTION: Vec2 = Vec2::new(-0.5, -0.5);

// WALLS
const WALL_THICKNESS: f32 = 15.0;
const LEFT_RIGHT_WALL_DIMS: Vec2 = Vec2::new(WALL_THICKNESS, SCREEN_HEIGHT - WALL_THICKNESS);
const TOP_BOTTOM_WALL_DIMS: Vec2 = Vec2::new(SCREEN_WIDTH - WALL_THICKNESS, WALL_THICKNESS);

const LEFT_WALL_POS: Vec2 = Vec2::new(WALL_THICKNESS - SCREEN_WIDTH / 2.0, 0.0);
const RIGHT_WALL_POS: Vec2 = Vec2::new(SCREEN_WIDTH / 2.0 - WALL_THICKNESS, 0.0);
const TOP_WALL_POS: Vec2 = Vec2::new(0.0, SCREEN_HEIGHT / 2.0 - WALL_THICKNESS);
const BOTTOM_WALL_POS: Vec2 = Vec2::new(0.0, -SCREEN_HEIGHT / 2.0 + WALL_THICKNESS);

const TOP_BOUND: f32 = TOP_WALL_POS.y - WALL_THICKNESS / 2.0 - PADDLE_DIMS.y / 2.0;
const BOTTOM_BOUND: f32 = BOTTOM_WALL_POS.y + WALL_THICKNESS / 2.0 + PADDLE_DIMS.y / 2.0;

// DOTTED LINE
const LINE_DIMS: Vec2 = Vec2::new(5.0, 20.0);
const NUM_DOTTED_LINES: i32 = 10;
// UPDATE TICK
const TIME_STEP: f32 = 1.0 / 72.0;
const PLAYER_PADDLE_SPEED: f32 = 500.0;

// DIFFICULTY
const AI_PADDLE_BASE_SPEED: f32 = 250.0;
const DIFFICULTY: f32 = 1.0;

// COLORS
const BACKGROUND_COLOR: Color = Color::BLACK;
const WALL_COLOR: Color = Color::WHITE;

// SCOREBOARD
const SCOREBOARD_FONT_SIZE: f32 = 40.0;
const SCOREBOARD_FONT_COLOR: Color = Color::GREEN;
const SCOREBOARD_TEXT_PADDING: f32 = 25.0;

// EVENTS
#[derive(Event, Default)]
pub struct CollisionEvent;

#[derive(Event, Default)]
pub struct LeftCollisionEvent {
    pub puck_position: Vec2,
    pub puck_direction: Vec2,
}

#[derive(Event, Default)]
pub struct GoalEvent {
    pub is_left_goal: bool,
}

// COMPONENTS
#[derive(Component)]
pub struct Collider;

#[derive(Component)]
pub struct Paddle;

#[derive(Component, Deref, DerefMut)]
pub struct Velocity(Vec2);

#[derive(Component)]
pub struct Goal;

#[derive(Component)]
pub struct Left;
#[derive(Component)]
pub struct Right;

#[derive(Component)]
pub struct Ai {
    pub y_target: f32,
}

// SOUNDS
#[derive(Resource)]
struct CollisionSound(Handle<AudioSource>);

fn main() {
    App::new()
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Unbeatable Pong".to_string(),
                resolution: (SCREEN_WIDTH, SCREEN_HEIGHT).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(Scoreboard {
            left_score: 0,
            right_score: 0,
        })
        .add_event::<CollisionEvent>()
        .add_event::<LeftCollisionEvent>()
        .add_event::<GoalEvent>()
        .insert_resource(Time::<Fixed>::from_seconds(TIME_STEP as f64))
        .add_systems(Startup, (setup, setup_scoreboard, setup_assets))
        .add_systems(
            FixedUpdate,
            (
                move_left_paddle.before(check_collisions),
                ai_move_right_paddle.before(check_collisions),
                apply_velocity.before(check_collisions),
                check_collisions,
                on_goal_scored,
                play_collision_sound.after(check_collisions),
                set_ai_target,
            ),
        )
        .run();
}

// SYSTEMS

// Startup Systems
fn setup(mut commands: Commands) {
    setup_camera(&mut commands);
    setup_walls(&mut commands);
    setup_paddles(&mut commands);
    setup_puck(&mut commands);
    setup_dotted_line(&mut commands);
}

fn setup_camera(commands: &mut Commands) {
    commands.spawn(Camera2dBundle::default());
}
fn setup_walls(commands: &mut Commands) {
    commands
        .spawn(RectBundle::new(LEFT_WALL_POS, LEFT_RIGHT_WALL_DIMS))
        .insert(Goal)
        .insert(Left);
    commands
        .spawn(RectBundle::new(RIGHT_WALL_POS, LEFT_RIGHT_WALL_DIMS))
        .insert(Goal);
    commands.spawn(RectBundle::new(TOP_WALL_POS, TOP_BOTTOM_WALL_DIMS));
    commands.spawn(RectBundle::new(BOTTOM_WALL_POS, TOP_BOTTOM_WALL_DIMS));
}

fn setup_paddles(commands: &mut Commands) {
    commands
        .spawn(RectBundle::new(LEFT_PADDLE_POS, PADDLE_DIMS))
        .insert(Paddle)
        .insert(Left);
    commands
        .spawn(RectBundle::new(RIGHT_PADDLE_POS, PADDLE_DIMS))
        .insert(Paddle)
        .insert(Right)
        .insert(Ai { y_target: 0.0 });
}

fn setup_puck(commands: &mut Commands) {
    commands
        .spawn(RectBundle::new(PUCK_SPAWN_POS, PUCK_DIMS))
        .insert(Velocity(INITIAL_PUCK_DIRECTION.normalize() * PUCK_SPEED));
}

fn setup_dotted_line(commands: &mut Commands) {
    let increment: f32 = SCREEN_HEIGHT / (NUM_DOTTED_LINES as f32);
    let bottom: f32 = -SCREEN_HEIGHT / 2.0 + LINE_DIMS.y + WALL_THICKNESS;
    for y_index in 0..NUM_DOTTED_LINES {
        commands.spawn(sprite_bundle_from_pos_size(
            Vec2::new(0.0, y_index as f32 * increment + bottom),
            LINE_DIMS,
        ));
    }
}

fn setup_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(CollisionSound(asset_server.load("sounds/blipSelect.ogg")));
}

// Game Logic Systems
fn apply_velocity(mut query: Query<(&mut Transform, &Velocity), With<Velocity>>) {
    for (mut transform, velocity) in &mut query {
        transform.translation.x += velocity.x * TIME_STEP;
        transform.translation.y += velocity.y * TIME_STEP;
    }
}

fn move_left_paddle(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, (With<Paddle>, With<Left>)>,
) {
    let mut paddle_transform = query.single_mut();
    let mut direction = 0.0;

    if keyboard_input.pressed(KeyCode::ArrowUp) {
        direction += 1.0;
    }

    if keyboard_input.pressed(KeyCode::ArrowDown) {
        direction -= 1.0;
    }

    let new_paddle_position =
        paddle_transform.translation.y + direction * PLAYER_PADDLE_SPEED * TIME_STEP;

    // Keep the paddle movement in bounds

    paddle_transform.translation.y = new_paddle_position.clamp(BOTTOM_BOUND, TOP_BOUND);
}

fn set_ai_target(
    mut left_collision_events: EventReader<LeftCollisionEvent>,
    mut ai_query: Query<&mut Ai, With<Paddle>>,
) {
    // On CollisionEvent, set a target for the ai controlled right paddle
    if let Some(collision) = left_collision_events.read().last() {
        let mut ai = ai_query.single_mut();

        ai.y_target = recursive_solve_right_wall_intercept(
            collision.puck_position,
            collision.puck_direction,
            0,
        )
        .y;
    }
}

fn recursive_solve_right_wall_intercept(pos: Vec2, dir: Vec2, bounces: usize) -> Vec2 {
    /// Recursively find an intercept on the right most wall
    ///
    /// # Arguments
    ///  * `pos` - position vector
    ///  * `dir` - direction vector

    // Base case MAX bounces exceeded
    const MAX_BOUNCES: usize = 5;
    if bounces >= MAX_BOUNCES {
        return pos;
    }
    // Base case 2, vertical contact point within bounds
    let cy: f32 = (RIGHT_PADDLE_POS.x - pos.x) * (dir.y / dir.x) + pos.y;
    if cy > BOTTOM_WALL_POS.y && cy < TOP_WALL_POS.y {
        return Vec2::new(RIGHT_PADDLE_POS.x, cy);
    }

    // Calculate vertical impact position
    let tby;
    if dir.y >= 0.0 {
        tby = TOP_WALL_POS.y;
    } else {
        tby = BOTTOM_WALL_POS.y;
    }

    // Recurse at point of impact if a vertical collision occurs
    // in the future
    let cx = (tby - pos.y) * (dir.x / dir.y) + pos.x;
    if cx < RIGHT_PADDLE_POS.x {
        return recursive_solve_right_wall_intercept(
            Vec2::new(cx, tby),
            Vec2::new(dir.x, -dir.y),
            bounces + 1,
        );
    }

    return pos;
}

fn ai_move_right_paddle(
    ai_query: Query<&Ai, (With<Paddle>, With<Ai>)>,
    mut paddle_query: Query<&mut Transform, (With<Paddle>, With<Right>)>,
) {
    let mut paddle_transform = paddle_query.single_mut();
    let ai_data = ai_query.single();

    if ai_data.y_target > paddle_transform.translation.y + PUCK_DIMS.y / 2.0 {
        paddle_transform.translation.y = f32::min(
            paddle_transform.translation.y + AI_PADDLE_BASE_SPEED * TIME_STEP * DIFFICULTY,
            TOP_BOUND,
        );
    } else if ai_data.y_target < paddle_transform.translation.y - PUCK_DIMS.y / 2.0 {
        paddle_transform.translation.y = f32::max(
            paddle_transform.translation.y - AI_PADDLE_BASE_SPEED * TIME_STEP * DIFFICULTY,
            BOTTOM_BOUND,
        );
    }
}

fn setup_scoreboard(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(
            TextBundle::from_sections([TextSection::new(
                "0",
                TextStyle {
                    font: asset_server.load("fonts/Arame-Bold.ttf"),
                    font_size: SCOREBOARD_FONT_SIZE,
                    color: SCOREBOARD_FONT_COLOR,
                },
            )])
            .with_style(Style {
                position_type: PositionType::Absolute,
                top: Val::Px(SCOREBOARD_TEXT_PADDING),
                left: Val::Px(SCREEN_WIDTH / 2.0 - 2.4*SCOREBOARD_TEXT_PADDING),
                ..default()
            }),
        )
        .insert(Left);

    commands
        .spawn(
            TextBundle::from_sections([TextSection::new(
                "0",
                TextStyle {
                    font: asset_server.load("fonts/Arame-Bold.ttf"),
                    font_size: SCOREBOARD_FONT_SIZE,
                    color: SCOREBOARD_FONT_COLOR,
                },
            )])
            .with_style(Style {
                position_type: PositionType::Absolute,
                top: Val::Px(SCOREBOARD_TEXT_PADDING),
                left: Val::Px(SCREEN_WIDTH / 2.0 + SCOREBOARD_TEXT_PADDING),
                ..default()
            }),
        )
        .insert(Right);
}

fn on_goal_scored(
    mut goal_events: EventReader<GoalEvent>,
    mut scoreboard: ResMut<Scoreboard>,
    mut left_query: Query<&mut Text, (With<Left>, Without<Right>)>,
    mut right_query: Query<&mut Text, (With<Right>, Without<Left>)>,
) {
    for goal_event in goal_events.read() {
        if goal_event.is_left_goal {
            scoreboard.right_score += 1;
            let mut right_text: Mut<Text> = right_query.single_mut();
            right_text.sections[0].value = scoreboard.right_score.to_string();
        } else {
            scoreboard.left_score += 1;
            let mut left_text = left_query.single_mut();
            left_text.sections[0].value = scoreboard.left_score.to_string();
        }
    }
}

fn check_collisions(
    mut collision_events: EventWriter<CollisionEvent>,
    mut left_collision_events: EventWriter<LeftCollisionEvent>,

    mut goal_events: EventWriter<GoalEvent>,
    mut mover_query: Query<(&mut Transform, &mut Velocity), With<Velocity>>,
    collider_query: Query<
        (&Transform, Option<&Goal>, Option<&Left>),
        (With<Collider>, Without<Velocity>),
    >,
) {
    for (mover_transform, mut mover_velocity) in &mut mover_query {
        for (collider_transform, goal, left) in &collider_query {
            let collision = check_aabb_collision(
                mover_transform.translation,
                mover_transform.scale.truncate(),
                collider_transform.translation,
                collider_transform.scale.truncate(),
            );
            if let Some(collision) = collision {
                collision_events.send_default();

                if let Some(_) = goal {
                    goal_events.send(GoalEvent {
                        is_left_goal: collider_transform.translation.x < 0.0,
                    });
                }

                // reflect the puck when it collides
                let mut reflect_x = false;
                let mut reflect_y = false;

                // only reflect if the puck's velocity is going in the opposite direction of the
                // collision
                match collision {
                    Collision::Left => reflect_x = mover_velocity.x > 0.0,
                    Collision::Right => reflect_x = mover_velocity.x < 0.0,
                    Collision::Top => reflect_y = mover_velocity.y < 0.0,
                    Collision::Bottom => reflect_y = mover_velocity.y > 0.0,
                    Collision::Inside => { /* do nothing */ }
                }

                // reflect velocity on the x-axis if we hit something on the x-axis
                if reflect_x {
                    mover_velocity.x = -mover_velocity.x;
                }

                // reflect velocity on the y-axis if we hit something on the y-axis
                if reflect_y {
                    mover_velocity.y = -mover_velocity.y;
                }

                if let Some(_) = left {
                    left_collision_events.send(LeftCollisionEvent {
                        puck_position: mover_transform.translation.truncate(),
                        puck_direction: mover_velocity.normalize(),
                    });
                }
            }
        }
    }
}

fn play_collision_sound(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    //audio: Res<Audio>,
    sound: Res<CollisionSound>,
) {
    // Play a sound once per frame if a collision occurred.
    if !collision_events.is_empty() {
        // This prevents events staying active on the next frame.
        collision_events.clear();
        commands.spawn(AudioBundle {
            source: sound.0.clone(),
            // auto-despawn the entity when playback finishes
            settings: PlaybackSettings::DESPAWN,
        });
        //audio.play(sound.0.clone());
    }
}

// BUNDLES and Resources

#[derive(Resource)]
struct Scoreboard {
    left_score: usize,
    right_score: usize,
}

#[derive(Bundle)]
struct RectBundle {
    //#[bundle]
    sprite_bundle: SpriteBundle,
    collider: Collider,
}

impl RectBundle {
    fn new(position: Vec2, size: Vec2) -> RectBundle {
        RectBundle {
            sprite_bundle: sprite_bundle_from_pos_size(position, size),
            collider: Collider,
        }
    }
}

fn sprite_bundle_from_pos_size(position: Vec2, size: Vec2) -> SpriteBundle {
    SpriteBundle {
        transform: Transform {
            translation: position.extend(0.0),
            scale: size.extend(0.0),
            ..default()
        },
        sprite: Sprite {
            color: WALL_COLOR,
            ..default()
        },
        ..default()
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum Collision {
    Left,
    Right,
    Top,
    Bottom,
    Inside,
}

struct CollisionBox {
    pub top: f32,
    pub bottom: f32,
    pub left: f32,
    pub right: f32,
}

impl CollisionBox {
    pub fn new(pos: Vec3, size: Vec2) -> Self {
        Self {
            top: pos.y + size.y / 2.,
            bottom: pos.y - size.y / 2.,
            left: pos.x - size.x / 2.,
            right: pos.x + size.x / 2.,
        }
    }
}

// From https://github.com/bevyengine/bevy/blob/6a3b059db917999b15ca032a4cab8cd31569b896/crates/bevy_sprite/src/collide_aabb.rs
fn check_aabb_collision(a_pos: Vec3, a_size: Vec2, b_pos: Vec3, b_size: Vec2) -> Option<Collision> {
    let a = CollisionBox::new(a_pos, a_size);
    let b = CollisionBox::new(b_pos, b_size);

    // check to see if the two rectangles are intersecting
    if a.left < b.right && a.right > b.left && a.bottom < b.top && a.top > b.bottom {
        // check to see if we hit on the left or right side
        let (x_collision, x_depth) = if a.left < b.left && a.right > b.left && a.right < b.right {
            (Collision::Left, b.left - a.right)
        } else if a.left > b.left && a.left < b.right && a.right > b.right {
            (Collision::Right, a.left - b.right)
        } else {
            (Collision::Inside, -f32::INFINITY)
        };

        // check to see if we hit on the top or bottom side
        let (y_collision, y_depth) = if a.bottom < b.bottom && a.top > b.bottom && a.top < b.top {
            (Collision::Bottom, b.bottom - a.top)
        } else if a.bottom > b.bottom && a.bottom < b.top && a.top > b.top {
            (Collision::Top, a.bottom - b.top)
        } else {
            (Collision::Inside, -f32::INFINITY)
        };

        // if we had an "x" and a "y" collision, pick the "primary" side using penetration depth
        if y_depth.abs() < x_depth.abs() {
            Some(y_collision)
        } else {
            Some(x_collision)
        }
    } else {
        None
    }
}
