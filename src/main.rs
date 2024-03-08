use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use world::DullWorld;

mod world;

#[derive(Resource)]
struct StepTimer(Timer);

#[derive(Component)]
struct DullCell(Rectangle);

fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let dull_world = DullWorld::from_config(vec![
        vec![0, 0, 0, 0, 0],
        vec![0, 0, 1, 0, 0],
        vec![0, 0, 1, 0, 0],
        vec![0, 0, 1, 0, 0],
        vec![0, 0, 0, 0, 0],
    ])
    .expect("initial world creation to succeed");

    for (row_index, row) in dull_world.get_grid().iter().enumerate() {
        for (col_index, cell) in row.iter().enumerate() {
            commands.spawn(MaterialMesh2dBundle {
                mesh: Mesh2dHandle(meshes.add(Rectangle::new(10., 10.))),
                material: materials.add(if *cell == 0 {
                    Color::PURPLE
                } else {
                    Color::GREEN
                }),
                transform: Transform::from_xyz(row_index as f32 * 10., col_index as f32 * 10., 0.0),
                ..default()
            });
        }
    }

    commands.spawn(dull_world);
    commands.spawn(Camera2dBundle::default());
}

fn update_world(time: Res<Time>, mut timer: ResMut<StepTimer>, mut query: Query<&mut DullWorld>) {
    if timer.0.tick(time.delta()).just_finished() {
        for mut dull_world in &mut query {
            dull_world.step();
        }
    }
}

fn render_world() {}

fn main() {
    App::new()
        .insert_resource(StepTimer(Timer::from_seconds(0.5, TimerMode::Repeating)))
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup_world)
        .add_systems(Update, update_world)
        .add_systems(PostUpdate, render_world)
        .run();
}
