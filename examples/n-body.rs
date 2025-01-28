use bevy::color::palettes::css::*;
use bevy::core_pipeline::core_2d::Camera2d;
use bevy::ecs::component::Component;
use bevy::math::primitives::Circle;
use bevy::pbr::{MeshMaterial3d, StandardMaterial};
use bevy::render::mesh::Mesh3d;
use bevy::{
    app::{App, Startup, Update},
    asset::Assets,
    ecs::{
        query::With,
        system::{Commands, Query, Res, ResMut},
    },
    math::{Vec2, Vec3},
    prelude::default,
    reflect::TypePath,
    render::{mesh::Mesh, render_resource::ShaderRef},
    sprite::ColorMaterial,
    time::Time,
    transform::components::Transform,
    window::{PrimaryWindow, Window},
    DefaultPlugins,
};
use bevy_simple_compute::prelude::*;
use bytemuck::{Pod, Zeroable};

const G: f32 = 10.;
const NUM_BODIES: usize = 4;

#[derive(ShaderType, Pod, Zeroable, Clone, Copy)]
#[repr(C)]
struct Body {
    mass: f32,
    pos: Vec2,
    vel: Vec2,
    acc: Vec2,
}

#[derive(TypePath)]
struct NBodyInteractShader;

impl ComputeShader for NBodyInteractShader {
    fn shader() -> ShaderRef {
        "shaders/n-body_interact.wgsl".into()
    }
}

#[derive(TypePath)]
struct NBodyIntegrateShader;

impl ComputeShader for NBodyIntegrateShader {
    fn shader() -> ShaderRef {
        "shaders/n-body_integrate.wgsl".into()
    }
}

pub struct NBodyWorker;

impl ComputeWorker for NBodyWorker {
    fn build(world: &mut bevy::prelude::World) -> AppComputeWorker<Self> {
        let mut initial_bodies_data = Vec::with_capacity(NUM_BODIES);

        for _ in 0..NUM_BODIES {
            initial_bodies_data.push(Body {
                mass: 250.,
                pos: Vec2::ZERO,
                acc: Vec2::ZERO,
                vel: Vec2::ZERO,
            })
        }

        AppComputeWorkerBuilder::new(world)
            .add_uniform("g", &G)
            .add_uniform("delta_time", &0.004f32)
            .add_staging("bodies_src", &initial_bodies_data)
            .add_staging("bodies_dst", &initial_bodies_data)
            .add_pass::<NBodyInteractShader>(
                [NUM_BODIES as u32, 1, 1],
                &["delta_time", "bodies_src", "bodies_dst"],
            )
            .add_swap("bodies_src", "bodies_dst")
            .build()
    }
}

#[derive(Component)]
struct BodyEntity(pub usize);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(AppComputePlugin)
        .add_plugins(AppComputeWorkerPlugin::<NBodyWorker>::default())
        .add_systems(Startup, setup)
        .add_systems(Update, move_bodies)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(Camera2d::default());

    for i in 0..NUM_BODIES {
        commands.spawn((
            Mesh3d(meshes.add(Circle::new(10.))),
            MeshMaterial3d(materials.add(StandardMaterial::from_color(GREEN))),
            Transform::from_translation(Vec3::new(-100., 100., 0.)),
            BodyEntity(i as usize),
        ));
    }
}

fn move_bodies(
    time: Res<Time>,
    mut worker: ResMut<AppComputeWorker<NBodyWorker>>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    mut q_bodies: Query<(&mut Transform, &BodyEntity)>,
) {
    if !worker.ready() {
        return;
    }

    let window = q_window.single();

    let bodies = worker.read_vec::<Body>("body_dst");
    worker.write("delta_time", &time.delta_secs());

    q_bodies
        .par_iter_mut()
        .for_each(|(mut transform, body_entity)| {
            let world_pos = Vec2::new(
                (window.width() / 2.) * (bodies[body_entity.0].pos.x),
                (window.height() / 2.) * (bodies[body_entity.0].pos.y),
            );

            transform.translation = world_pos.extend(0.);
        });
}

/*
fn update_pos(
    mut q_bodies: Query<(&mut Transform, &Body)>
) {
    for (mut transform, body) in &mut q_bodies {
        transform.translation += body.vel.extend(0.);
    }
}

fn simulate(
    time: Res<Time>,
    mut q_bodies: Query<(&Transform, &mut Body)>
) {

    let mut combinaisons = q_bodies.iter_combinations_mut();
    let dt_sq = time.delta_seconds() * time.delta_seconds();

    while let Some([(trans_a, mut body_a), (trans_b, mut body_b)]) = combinaisons.fetch_next() {
        let delta = trans_b.translation.truncate() - trans_a.translation.truncate();
        let distance_sq = delta.length_squared();

        let f = G / distance_sq;
        let force_unit_mass = delta * f;
        body_a.acc += force_unit_mass * body_b.mass;
        body_b.acc -= force_unit_mass * body_a.mass;
    }

    for (_, mut body) in &mut q_bodies {
        let acc = body.acc;
        body.vel += acc * dt_sq;
        body.acc = Vec2::ZERO;
    }

}
*/
