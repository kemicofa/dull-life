use bevy::{
    prelude::*,
    render::camera::ScalingMode,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use bevy_prng::WyRand;
use bevy_rand::prelude::EntropyPlugin;
use bevy_rand::prelude::GlobalEntropy;
use rand_core::RngCore;

use plugins::dl_window::DullLifeWindowPlugin;
use world::DullWorld;

mod plugins;
mod world;

const ROWS: usize = 120;
const COLS: usize = 240;
const THRESHOLD_FOR_INITIAL_LIFE: u32 = (u32::MAX as f32 * 0.3) as u32;

#[derive(Resource)]
struct StepTimer(Timer);

#[derive(Resource)]
struct CellDeadColor(Handle<ColorMaterial>);

#[derive(Resource)]
struct CellLiveColor(Handle<ColorMaterial>);

#[derive(Component)]
struct Cell {
    row_index: usize,
    col_index: usize,
}

fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut rng: ResMut<GlobalEntropy<WyRand>>,
) {
    let live_color_handle = materials.add(Color::GREEN);
    let dead_color_handle = materials.add(Color::PURPLE);

    let cell_live_color = CellLiveColor(live_color_handle.clone());
    let cell_dead_color = CellDeadColor(dead_color_handle.clone());

    commands.insert_resource(cell_live_color);
    commands.insert_resource(cell_dead_color);

    let mut random_world: Vec<Vec<u8>> = vec![vec![0; ROWS]; COLS];
    for i in 0..COLS {
        for j in 0..ROWS {
            let random_number = rng.next_u32();
            if random_number < THRESHOLD_FOR_INITIAL_LIFE {
                random_world[i][j] = 1;
            }
        }
    }
    let dull_world =
        DullWorld::from_config(random_world).expect("initial world creation to succeed");

    let (row_len, col_len) = dull_world.dimensions();

    for row_index in 0..row_len {
        for col_index in 0..col_len {
            commands.spawn((
                MaterialMesh2dBundle {
                    mesh: Mesh2dHandle(meshes.add(Rectangle::new(10., 10.))),
                    material: if dull_world.is_live(row_index, col_index) {
                        live_color_handle.clone()
                    } else {
                        dead_color_handle.clone()
                    },
                    transform: Transform::from_xyz(
                        (row_index as f32 - row_len as f32 / 2.) * 10.,
                        (col_index as f32 - col_len as f32 / 2.) * 10.,
                        0.0,
                    ),
                    ..default()
                },
                Cell {
                    row_index,
                    col_index,
                },
            ));
        }
    }

    commands.spawn(dull_world);

    // Camera
    let mut camera = Camera2dBundle::default();
    camera.projection.scaling_mode = ScalingMode::FixedVertical(1000.0);
    commands.spawn(camera);
}

fn update_world(time: Res<Time>, mut timer: ResMut<StepTimer>, mut query: Query<&mut DullWorld>) {
    if timer.0.tick(time.delta()).just_finished() {
        for mut dull_world in &mut query {
            dull_world.step();
        }
    }
}

fn render_world(
    mut q_entities: Query<(&mut Handle<ColorMaterial>, &Cell)>,
    q_dull_world: Query<&DullWorld>,
    q_dead_cell_color: Res<CellDeadColor>,
    q_live_cell_color: Res<CellLiveColor>,
) {
    let dull_world = q_dull_world.iter().next().unwrap();
    for (mut entity, cell) in q_entities.iter_mut() {
        if dull_world.is_live(cell.row_index, cell.col_index) {
            *entity = q_dead_cell_color.0.clone();
        } else {
            *entity = q_live_cell_color.0.clone();
        };
    }
}

fn main() {
    App::new()
        .insert_resource(StepTimer(Timer::from_seconds(0.2, TimerMode::Repeating)))
        .add_plugins((DullLifeWindowPlugin, EntropyPlugin::<WyRand>::default()))
        .add_systems(Startup, setup_world)
        .add_systems(Update, update_world)
        .add_systems(PostUpdate, render_world)
        .run();
}
