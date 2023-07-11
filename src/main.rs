use std::{collections::VecDeque, iter::zip};

use bevy::{
    prelude::*, sprite::MaterialMesh2dBundle, time::common_conditions::on_timer, utils::Duration,
};

use rand::{seq::IteratorRandom, thread_rng};

const FONT: &'static str = "fonts/FiraMonoNerdFont-Bold.otf";

const STEP: i16 = 10;
const WALL_SIZE: i16 = 200;
const WALL_POS: f32 = (WALL_SIZE + STEP) as f32;
const STRETCH: f32 = 2. * WALL_POS;
const THICKNESS: f32 = 5.;

const SCORE_SIZE: f32 = 20.;
const SCOREBOARD_FONT_SIZE: f32 = 40.0;
const SCORE_COLOR: Color = Color::RED;

const SNAKE_COLOR: Color = Color::GREEN;
const WALL_COLOR: Color = Color::BLUE;

#[derive(Resource)]
struct Scoreboard {
    score: usize,
}

#[derive(Component)]
struct ScoreText;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States, SystemSet)]
enum AppState {
    #[default]
    InGame,
    Lost,
}

#[derive(Component, Debug)]
struct Body;

#[derive(Component, Debug)]
struct Head;

#[derive(Component, Debug)]
struct Apple;

#[derive(Debug, Default, Copy, Clone)]
struct Direction {
    x: i16,
    y: i16,
}

#[derive(Resource, Debug, Default)]
struct Snake {
    ids: Vec<Entity>,
    dirs: VecDeque<Direction>,
}

impl Snake {
    fn add_entity(&mut self, entity: Entity, dir: Direction) {
        self.ids.push(entity);
        self.dirs.push_back(dir);
    }

    fn clear(&mut self) {
        self.ids.clear();
        self.dirs.clear();
    }
}

#[derive(Resource, Debug, Default)]
struct SnakeLength(usize);

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins);

    app.insert_resource(ClearColor(Color::BLACK));
    app.insert_resource(Snake::default());
    app.insert_resource(Scoreboard { score: 0 });

    app.add_systems(Startup, (setup, setup_items));

    app.add_state::<AppState>()
        .add_systems(
            Update,
            (
                wall_collision,
                change_direction.after(wall_collision),
                move_snake
                    .run_if(on_timer(Duration::from_secs_f32(0.10)))
                    .before(change_direction)
                    .after(wall_collision),
                eat_apple.after(move_snake),
                check_lost.after(move_snake),
            )
                .run_if(in_state(AppState::InGame)),
        )
        .add_systems(
            Update,
            (
                clear_map.before(setup_items),
                setup_items.after(clear_map),
                enter_game.after(clear_map).after(setup_items),
            )
                .run_if(in_state(AppState::Lost)),
        );

    app.run();
}

fn get_square(
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    color: Color,
) -> MaterialMesh2dBundle<ColorMaterial> {
    MaterialMesh2dBundle {
        mesh: meshes.add(Mesh::from(shape::Quad::default())).into(),
        transform: Transform::default().with_scale(Vec3::splat(STEP as f32)),
        material: materials.add(ColorMaterial::from(color)),
        ..Default::default()
    }
}

fn setup_items(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut snake: ResMut<Snake>,
) {
    let square = get_square(&mut meshes, &mut materials, SNAKE_COLOR);
    let wall_size = WALL_SIZE as f32;
    let step = STEP as f32;

    // spawn snake in lower right corner
    let head_id = commands
        .spawn((
            Body {},
            Head {},
            SpatialBundle {
                transform: Transform::from_xyz(wall_size - step, -wall_size, 1.),
                ..Default::default()
            },
        ))
        .with_children(|parent| {
            // spawn a square shape as snake head
            parent.spawn(square.clone());
        })
        .id();

    let dir = Direction { x: STEP, y: 0 };
    snake.add_entity(head_id, dir);

    // spawn square shapes as snake body
    for i in 2..((wall_size / 6.).round() as usize) {
        let tail_id = commands
            .spawn((
                Body {},
                SpatialBundle {
                    transform: Transform::from_xyz(wall_size - i as f32 * step, -wall_size, 1.),
                    ..Default::default()
                },
            ))
            .with_children(|parent| {
                parent.spawn(square.clone());
            })
            .id();
        snake.add_entity(tail_id, dir);
    }

    let red_square = get_square(&mut meshes, &mut materials, Color::RED);

    // spawn apple in center
    commands
        .spawn((Apple {}, SpatialBundle::default()))
        .with_children(|parent| {
            parent.spawn(red_square);
        });
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, scoreboard: Res<Scoreboard>) {
    commands.spawn(Camera2dBundle::default());

    let wall = |position: Vec2, size: Vec2| SpriteBundle {
        transform: Transform {
            translation: position.extend(0.0),
            scale: size.extend(1.0),
            ..Default::default()
        },
        sprite: Sprite {
            color: WALL_COLOR,
            ..Default::default()
        },
        ..Default::default()
    };

    // Spawn square walls
    let hor_wall = Vec2::new(STRETCH + THICKNESS, THICKNESS);
    let vert_wall = Vec2::new(THICKNESS, STRETCH + THICKNESS);
    commands.spawn(wall(Vec2::new(0., WALL_POS), hor_wall)); // top
    commands.spawn(wall(Vec2::new(0., -WALL_POS), hor_wall)); // bottom
    commands.spawn(wall(Vec2::new(WALL_POS, 0.), vert_wall)); // right
    commands.spawn(wall(Vec2::new(-WALL_POS, 0.), vert_wall)); // left

    commands.spawn((
        ScoreText,
        Text2dBundle {
            text: Text {
                sections: vec![TextSection::new(
                    scoreboard.score.to_string(),
                    TextStyle {
                        font: asset_server.load(FONT),
                        font_size: SCOREBOARD_FONT_SIZE,
                        color: SCORE_COLOR,
                    },
                )],
                ..Default::default()
            },
            transform: Transform::from_xyz(0., WALL_POS + SCORE_SIZE + STEP as f32 + THICKNESS, 1.)
                .with_scale(Vec3::splat(1.0)),
            ..Default::default()
        },
    ));
}

fn check_lost(
    mut app_state: ResMut<NextState<AppState>>,
    head_query: Query<&Transform, With<Head>>,
    body_pos_query: Query<&Transform, (With<Body>, Without<Head>)>,
) {
    let head_pos = head_query.single();
    for body_pos in body_pos_query.iter() {
        let diff = (head_pos.translation - body_pos.translation).abs();
        if diff.x <= f32::EPSILON && diff.y <= f32::EPSILON {
            app_state.set(AppState::Lost);
            return;
        }
    }
}

fn clear_map(
    mut commands: Commands,
    body_query: Query<(Entity, &Children), With<Body>>,
    apple_query: Query<(Entity, &Children), With<Apple>>,
    mut snake: ResMut<Snake>,
    mut scoreboard: ResMut<Scoreboard>,
    mut text_query: Query<&mut Text, With<ScoreText>>,
) {
    for (entity, children) in &body_query {
        commands.entity(entity).despawn();
        for &child in children {
            commands.entity(child).despawn();
        }
    }

    let (apple_entity, children) = apple_query.single();
    commands.entity(apple_entity).despawn();
    for &child in children {
        commands.entity(child).despawn();
    }

    snake.clear();

    scoreboard.score = 0;
    let mut text = text_query.single_mut();
    text.sections[0].value = scoreboard.score.to_string();
}

fn enter_game(mut app_state: ResMut<NextState<AppState>>) {
    app_state.set(AppState::InGame);
}

fn eat_apple(
    mut commands: Commands,
    head: Query<&Transform, With<Head>>,
    body: Query<&Transform, With<Body>>,
    mut apple: Query<&mut Transform, (With<Apple>, Without<Body>, Without<Head>)>,
    mut snake: ResMut<Snake>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut scoreboard: ResMut<Scoreboard>,
    mut text_query: Query<&mut Text, With<ScoreText>>,
) {
    let mut apple_pos = apple.single_mut();
    let head_pos = head.single();

    let diff = (apple_pos.translation - head_pos.translation).abs();
    if (diff.x <= f32::EPSILON) && (diff.y <= f32::EPSILON) {
        let black_square = get_square(&mut meshes, &mut materials, SNAKE_COLOR);

        let tail_id = snake.ids.last().unwrap().clone();
        let tail_dir = snake.dirs.back().unwrap().clone();

        let tail_pos = body.get(tail_id).unwrap().translation;

        let new_tail_id = commands
            .spawn((
                Body {},
                SpatialBundle {
                    transform: Transform::from_xyz(
                        tail_pos.x - tail_dir.x as f32,
                        tail_pos.y - tail_dir.y as f32,
                        1.,
                    ),
                    ..Default::default()
                },
            ))
            .with_children(|parent| {
                parent.spawn(black_square.clone());
            })
            .id();

        snake.add_entity(new_tail_id, tail_dir);

        let mut rng = thread_rng();
        let x_dist = (-WALL_SIZE..WALL_SIZE).step_by(STEP as usize).filter(|i| {
            body.iter()
                .map(|pos| pos.translation.x)
                .any(|x| *i != x as i16)
        });
        let y_dist = (-WALL_SIZE..WALL_SIZE).step_by(STEP as usize).filter(|i| {
            body.iter()
                .map(|pos| pos.translation.y)
                .any(|y| *i != y as i16)
        });

        apple_pos.translation.x = x_dist.choose(&mut rng).unwrap() as f32;
        apple_pos.translation.y = y_dist.choose(&mut rng).unwrap() as f32;

        let mut text = text_query.single_mut();
        scoreboard.score += 1;
        text.sections[0].value = scoreboard.score.to_string();
    }
}

fn change_direction(keyboard_input: Res<Input<KeyCode>>, mut parts: ResMut<Snake>) {
    let direction = &mut parts.dirs[0];
    if keyboard_input.pressed(KeyCode::Left) && direction.x == 0 {
        direction.x = -STEP;
        direction.y = 0;
    }
    if keyboard_input.pressed(KeyCode::Right) && direction.x == 0 {
        direction.x = STEP;
        direction.y = 0;
    }
    if keyboard_input.pressed(KeyCode::Up) && direction.y == 0 {
        direction.x = 0;
        direction.y = STEP;
    }
    if keyboard_input.pressed(KeyCode::Down) && direction.y == 0 {
        direction.x = 0;
        direction.y = -STEP;
    }
}

fn move_snake(
    mut query: Query<&mut Transform, (With<Body>, Without<Apple>)>,
    mut parts: ResMut<Snake>,
) {
    for (entity, dir) in zip(&parts.ids, &parts.dirs) {
        let mut pos = query.get_mut(*entity).unwrap();
        pos.translation.x += dir.x as f32;
        pos.translation.y += dir.y as f32;
    }

    parts.dirs.pop_back();
    let front = *parts.dirs.front().unwrap();
    parts.dirs.push_front(front);
}

fn wall_collision(mut snake: Query<&mut Transform, With<Body>>) {
    let wall_size = WALL_SIZE as f32;
    let limit = wall_size + STEP as f32;
    for mut pos in snake.iter_mut() {
        if pos.translation.x >= limit {
            pos.translation.x = -wall_size;
        }
        if pos.translation.x <= -limit {
            pos.translation.x = wall_size;
        }
        if pos.translation.y >= limit {
            pos.translation.y = -wall_size;
        }
        if pos.translation.y <= -limit {
            pos.translation.y = wall_size;
        }
    }
}
