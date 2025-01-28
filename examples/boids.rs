//! Example showing how to calculate boids data from compute shaders
//! For now they are stupid and just fly straight, need to fix this later on.
//! Reimplementation of https://github.com/gfx-rs/wgpu-rs/blob/master/examples/boids/main.rs

use bevy::color::palettes::css::*;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};

use bevy::{prelude::*, window::PrimaryWindow};

use bevy_simple_compute::prelude::*;
use bytemuck::{Pod, Zeroable};

use rand::distr::{Distribution, Uniform};

// Debug mode
//const NUM_BOIDS: u32 = 500;

// Release mode
const NUM_BOIDS: u32 = 2_000;

#[derive(ShaderType, Pod, Zeroable, Clone, Copy)]
#[repr(C)]
struct Params {
    speed: f32,
    rule_1_distance: f32,
    rule_2_distance: f32,
    rule_3_distance: f32,
    rule_1_scale: f32,
    rule_2_scale: f32,
    rule_3_scale: f32,
}

#[derive(ShaderType, Pod, Zeroable, Clone, Copy)]
#[repr(C)]
struct Boid {
    pos: Vec2,
    vel: Vec2,
}

#[derive(TypePath)]
struct BoidsShader;

impl ComputeShader for BoidsShader {
    fn shader() -> ShaderRef {
        "shaders/boids.wgsl".into()
    }
}

struct BoidWorker;

impl ComputeWorker for BoidWorker {
    fn build(world: &mut World) -> AppComputeWorker<Self> {
        let params = Params {
            speed: 0.5,
            rule_1_distance: 0.2,
            rule_2_distance: 0.025,
            rule_3_distance: 0.01,
            rule_1_scale: 0.08,
            rule_2_scale: 0.02,
            rule_3_scale: 0.01,
        };

        let mut initial_boids_data = Vec::with_capacity(NUM_BOIDS as usize);
        let mut rng = rand::rng();
        let unif = Uniform::new_inclusive(-1., 1.).expect("Couldn't create new Uniform rand distribution instance!");

        for _ in 0..NUM_BOIDS {
            initial_boids_data.push(Boid {
                pos: Vec2::new(unif.sample(&mut rng), unif.sample(&mut rng)),
                vel: Vec2::new(
                    unif.sample(&mut rng) * params.speed,
                    unif.sample(&mut rng) * params.speed,
                ),
            });
        }

        AppComputeWorkerBuilder::new(world)
            .add_uniform("params", &params)
            .add_uniform("delta_time", &0.004f32)
            .add_staging("boids_src", &initial_boids_data)
            .add_staging("boids_dst", &initial_boids_data)
            .add_pass::<BoidsShader>(
                [NUM_BOIDS, 1, 1],
                &["params", "delta_time", "boids_src", "boids_dst"],
            )
            .add_swap("boids_src", "boids_dst")
            .build()
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(AppComputePlugin)
        .add_plugins(AppComputeWorkerPlugin::<BoidWorker>::default())
        .insert_resource(ClearColor(DARK_GRAY.into()))
        .add_systems(Startup, setup)
        .add_systems(Update, move_entities)
        .run();
}

#[derive(Component)]
struct BoidEntity(pub usize);

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d::default());

    let boid_mesh = meshes.add(RegularPolygon::new(5., 3));
    let boid_material = materials.add(ColorMaterial::from_color(ANTIQUE_WHITE));

    // First boid in red, so we can follow it easily
    commands.spawn((
        BoidEntity(0),
        Mesh2d(boid_mesh.clone()),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(ORANGE_RED))),
    ));

    for i in 1..NUM_BOIDS {
        commands.spawn((
            BoidEntity(i as usize),
            Mesh2d(boid_mesh.clone()),
            MeshMaterial2d(boid_material.clone()),
        ));
    }
}

fn move_entities(
    time: Res<Time>,
    mut worker: ResMut<AppComputeWorker<BoidWorker>>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    mut q_boid: Query<(&mut Transform, &BoidEntity), With<BoidEntity>>,
) {
    if !worker.ready() {
        return;
    }

    let window = q_window.single();

    let boids = worker.read_vec::<Boid>("boids_dst");

    worker.write("delta_time", &time.delta_secs());

    q_boid
        .par_iter_mut()
        .for_each(|(mut transform, boid_entity)| {
            let world_pos = Vec2::new(
                (window.width() / 2.) * (boids[boid_entity.0].pos.x),
                (window.height() / 2.) * (boids[boid_entity.0].pos.y),
            );

            transform.translation = world_pos.extend(0.);
            transform.look_to(Vec3::Z, boids[boid_entity.0].vel.extend(0.));
        });
}
