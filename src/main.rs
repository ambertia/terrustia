use bevy::prelude::*;

// Dot's velocity in pixels per second
const DOT_VELOCITY: f32 = 60.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, setup)
        .add_systems(Update, handle_dot)
        .run();
}

#[derive(Component)]
struct Dot;

#[derive(Component)]
struct Velocity(Vec2);

// Move the dot around the screen based on keyboard input
fn handle_dot(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut dot_transform: Single<&mut Transform, With<Dot>>,
    time: Res<Time>,
) {
    let mut input_direction = Vec2::splat(0.0);

    // Check all the keys and modify the dot's input_direction accordingly
    if keyboard.pressed(KeyCode::KeyW) {
        input_direction.y += 1.0;
    }

    if keyboard.pressed(KeyCode::KeyA) {
        input_direction.x -= 1.0;
    }

    if keyboard.pressed(KeyCode::KeyS) {
        input_direction.y -= 1.0;
    }

    if keyboard.pressed(KeyCode::KeyD) {
        input_direction.x += 1.0;
    }

    println!("{}", input_direction);

    let distance_potential = time.delta_secs() * DOT_VELOCITY;
    dot_transform.translation.x += input_direction.x * distance_potential;
    dot_transform.translation.y += input_direction.y * distance_potential;
}

// Initialize all the stuff in the world
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);

    // Spawn the dot
    commands.spawn((
        Dot,
        Velocity,
        Mesh2d(meshes.add(Circle::default())),
        MeshMaterial2d(materials.add(Color::WHITE)),
        Transform::default().with_scale(Vec2::splat(10.0).extend(1.)),
    ));
}
