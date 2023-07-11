use std::{collections::VecDeque, iter::zip};

use bevy::{
    prelude::*, sprite::MaterialMesh2dBundle, time::common_conditions::on_timer, utils::Duration,
};
use rand::{seq::IteratorRandom, thread_rng};

const STEP: i16 = 10;
const WALL_SIZE: i16 = 200;

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
}

#[derive(Resource, Debug, Default)]
struct SnakeLength(usize);

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Snakers".to_string(),
            resolution: (700., 800.).into(),
            ..Default::default()
        }),
        ..Default::default()
    }));

    app.insert_resource(Snake::default());

    app.add_startup_system(setup);

    app.add_system(wall_collision);
    app.add_system(change_direction.before(wall_collision));
    app.add_system(
        move_snake
            .run_if(on_timer(Duration::from_secs_f32(0.15)))
            .after(change_direction)
            .after(wall_collision),
    );
    app.add_system(eat_apple.after(move_snake));

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
        ..default()
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut snake: ResMut<Snake>,
) {
    commands.spawn(Camera2dBundle::default());

    let wall_size = WALL_SIZE as f32;

    let black_square = get_square(&mut meshes, &mut materials, Color::BLACK);
    let red_square = get_square(&mut meshes, &mut materials, Color::RED);

    // spawn apple in center
    commands
        .spawn((Apple {}, SpatialBundle::default()))
        .with_children(|parent| {
            parent.spawn(red_square);
        });

    // spawn snake in lower right corner
    let head_id = commands
        .spawn((
            Body {},
            Head {},
            SpatialBundle {
                transform: Transform::from_xyz(wall_size, -wall_size, 1.),
                ..Default::default()
            },
        ))
        .with_children(|parent| {
            // spawn a square shape as snake head
            parent.spawn(black_square.clone());
        })
        .id();

    // spawn a square shape as snake body
    let tail_id = commands
        .spawn((
            Body {},
            SpatialBundle {
                transform: Transform::from_xyz(wall_size - STEP as f32, -wall_size, 1.),
                ..Default::default()
            },
        ))
        .with_children(|parent| {
            parent.spawn(black_square.clone());
        })
        .id();

    let dir = Direction { x: STEP, y: 0 };
    snake.add_entity(head_id, dir);
    snake.add_entity(tail_id, dir);
}

fn eat_apple(
    mut commands: Commands,
    head: Query<&Transform, With<Head>>,
    body: Query<&Transform, With<Body>>,
    mut apple: Query<&mut Transform, (With<Apple>, Without<Body>, Without<Head>)>,
    mut snake: ResMut<Snake>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut apple_pos = apple.single_mut();
    let head_pos = head.single();

    let diff = (apple_pos.translation - head_pos.translation).abs();
    if (diff.x <= f32::EPSILON) && (diff.y <= f32::EPSILON) {
        let black_square = get_square(&mut meshes, &mut materials, Color::BLACK);

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
