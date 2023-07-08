use bevy::{
    prelude::*, sprite::MaterialMesh2dBundle, time::common_conditions::on_timer, utils::Duration,
};
use rand::{seq::IteratorRandom, thread_rng};

const EPSILON: f32 = 1e-10;

#[derive(Component, Debug)]
struct Direction {
    x: i16,
    y: i16,
}

#[derive(Component)]
struct Snake;

#[derive(Component)]
struct Apple;

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

    app.add_startup_system(setup);
    app.add_system(change_direction);
    app.add_system(
        move_snake
            .run_if(on_timer(Duration::from_secs_f32(0.15)))
            .after(change_direction)
            .before(apple_collision),
    );
    app.add_system(apple_collision);
    app.add_system(
        wall_collision
            .before(apple_collision)
            .after(move_snake)
            .after(change_direction),
    );
    app.add_system(bevy::window::close_on_esc);

    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());

    commands
        .spawn((
            Snake {},
            Direction { x: 0, y: 0 },
            TransformBundle {
                local: Transform::from_xyz(-100., -100., 1.),
                ..Default::default()
            },
            VisibilityBundle::default(),
        ))
        .with_children(|parent| {
            parent
                .spawn(MaterialMesh2dBundle {
                    mesh: meshes.add(Mesh::from(shape::Quad::default())).into(),
                    transform: Transform::default().with_scale(Vec3::splat(10.)),
                    material: materials.add(ColorMaterial::from(Color::BLACK)),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(MaterialMesh2dBundle {
                        mesh: meshes.add(Mesh::from(shape::Quad::default())).into(),
                        transform: Transform::from_xyz(-1., 0., 1.),
                        material: materials.add(ColorMaterial::from(Color::BLACK)),
                        ..default()
                    });
                });
        });
    commands
        .spawn((
            Apple {},
            TransformBundle {
                local: Transform::from_xyz(0., 0., 0.),
                ..Default::default()
            },
            VisibilityBundle::default(),
        ))
        .with_children(|parent| {
            parent.spawn(MaterialMesh2dBundle {
                mesh: meshes.add(Mesh::from(shape::Quad::default())).into(),
                transform: Transform::default().with_scale(Vec3::splat(10.)),
                material: materials.add(ColorMaterial::from(Color::RED)),
                ..default()
            });
        });
}

fn wall_collision(mut snake: Query<&mut Transform, (With<Snake>, Without<Apple>)>) {
    let mut snake_pos = snake.single_mut();

    if snake_pos.translation.x >= 110. {
        snake_pos.translation.x = -100.;
    }
    if snake_pos.translation.x <= -110. {
        snake_pos.translation.x = 100.;
    }
    if snake_pos.translation.y >= 110. {
        snake_pos.translation.y = -100.;
    }
    if snake_pos.translation.y <= -110. {
        snake_pos.translation.y = 100.;
    }
}

fn apple_collision(
    mut commands: Commands,
    mut apple: Query<&mut Transform, (With<Apple>, Without<Snake>)>,
    mut snake: Query<&Transform, (With<Snake>, Without<Apple>)>,
) {
    let mut apple_pos = apple.single_mut();
    let snake_pos = snake.single_mut();

    let diff = (apple_pos.translation - snake_pos.translation).abs();
    if (diff.x <= EPSILON) && (diff.y <= EPSILON) {
        let mut rng = thread_rng();
        let dist = (-100..100).step_by(10);

        let mut new_x = dist.clone().choose(&mut rng).unwrap() as f32;
        let mut new_y = dist.clone().choose(&mut rng).unwrap() as f32;

        while (new_x - snake_pos.translation.x).abs() < EPSILON {
            new_x = dist.clone().choose(&mut rng).unwrap() as f32;
        }
        while (new_y - snake_pos.translation.y).abs() < EPSILON {
            new_y = dist.clone().choose(&mut rng).unwrap() as f32;
        }
        apple_pos.translation.x = new_x;
        apple_pos.translation.y = new_y;
    }
}

fn move_snake(mut snake: Query<(&mut Transform, &Direction), With<Snake>>) {
    let (mut snake_pos, direction) = snake.single_mut();

    snake_pos.translation.x += (direction.x * 10) as f32;
    snake_pos.translation.y += (direction.y * 10) as f32;
}

fn change_direction(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Direction, With<Snake>>,
) {
    let mut direction = query.single_mut();

    if keyboard_input.pressed(KeyCode::Left) && direction.x == 0 {
        direction.x = -1;
        direction.y = 0;
    }
    if keyboard_input.pressed(KeyCode::Right) && direction.x == 0 {
        direction.x = 1;
        direction.y = 0;
    }
    if keyboard_input.pressed(KeyCode::Up) && direction.y == 0 {
        direction.x = 0;
        direction.y = 1;
    }
    if keyboard_input.pressed(KeyCode::Down) && direction.y == 0 {
        direction.x = 0;
        direction.y = -1;
    }
}
