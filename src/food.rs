use bevy::prelude::*;
use avian2d::prelude::*;
use rand::Rng;

use crate::{GameLayer, Handlers, Hp};

pub struct FoodPlugin;

impl Plugin for FoodPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup_food)
            .add_systems(FixedUpdate, regrow_system)
        ;
    }
}


#[derive(Component, Clone)]
pub struct Food;

fn setup_food(
    mut commands: Commands,
    mut handlers: ResMut<Handlers>, 
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
){
    handlers.mesh_food = Some( meshes.add( Circle::new(3.) ) );
    handlers.material_food = Some( materials.add(Color::hsv(120., 0.2, 0.6)) );

    let mut rng = rand::rng();
    (0..1_000).into_iter().for_each(|_|{
        spawn_food(&mut commands, &handlers, 
            Transform::from_xyz(rng.random_range(-4_000_f32..4_000_f32), rng.random_range(-4_000_f32..4_000_f32), 0.),
            rng.random_range(0.7_f32..0.9_f32)
        );
    });
}

fn regrow_system(
    mut commands: Commands,
    handlers: ResMut<Handlers>, 
    entities: Query<&Food>,
){
    let l = entities.iter().len();
    if l < 6_000 {
        let mut rng = rand::rng();
        spawn_food(&mut commands, &handlers, 
            Transform::from_xyz(rng.random_range(-8_000_f32..8_000_f32), rng.random_range(-8_000_f32..8_000_f32), 0.),
            rng.random_range(0.2_f32..0.5_f32)
        );
    } 
}

fn spawn_food(commands: &mut Commands, handlers: &ResMut<Handlers>, transform: Transform, hp: f32){
    commands.spawn((
        Food,
        Hp(hp),
        Mesh2d( handlers.mesh_food.clone().unwrap() ),
        MeshMaterial2d( handlers.material_food.clone().unwrap() ),
        RigidBody::Static,
        Collider::circle(3.),
        CollisionLayers::new([GameLayer::Default, GameLayer::Foods], [GameLayer::Sectarii, GameLayer::Foods, GameLayer::Bullets]),
        transform
    ));
}
