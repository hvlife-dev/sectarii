use std::collections::HashMap;

use bevy::{math::ops::atan2, prelude::*};
use avian2d::{math::PI, prelude::*};

mod sectarii;
use sectarii::{Brain, SectariiPlugin};
mod food;
use food::{Food, FoodPlugin};
mod ui;
use ui::UiPlugin;

#[derive(PhysicsLayer, Default)]
enum GameLayer {
    #[default]
    Default,
    Sectarii,
    Foods,
    Bullets
}


#[derive(Component, Clone)]
struct Hp(f32);
fn hp_system(
    mut commands: Commands,
    mut entities: Query<(Entity, &mut Hp)>,
){
    entities.iter_mut().for_each(|(e, mut hp)| {
        hp.0 = hp.0.clamp(0., 2.);
        if hp.0 <= 0_f32 {
            commands.entity(e).despawn_recursive();
        }
    } );
}

#[derive(Component, Clone)]
struct Satiety(f32);
fn satiety_system(
    time: Res<Time>, 
    mut entities: Query<(&mut Hp, &mut Satiety), Without<Food>>
){
    entities.par_iter_mut().for_each(|(mut hp, mut satiety)| {
        satiety.0 -= time.delta_secs() / 45_f32;
        if satiety.0 <= 0_f32 {hp.0 -= time.delta_secs() / 15_f32}
        satiety.0 = satiety.0.clamp(0., 2.);
    } );
}

#[derive(Component, Clone)]
struct Stamina(f32);
fn stamina_system(
    time: Res<Time>, 
    mut entities: Query<(&mut Stamina, &mut Satiety, &mut Hp, &Brain)>
){
    entities.par_iter_mut().for_each(|(mut stamina, mut satiety, mut hp, brain)| {
        if brain.linvel < 10. { stamina.0 = (stamina.0 + time.delta_secs()).clamp(0., 2.);}
        else { 
            stamina.0 -= (brain.linvel - 9.).powi(2) * time.delta_secs() * 0.00075;
            if stamina.0 < 0. {
                if satiety.0 > 0. { satiety.0 += stamina.0 }
                else { hp.0 += stamina.0 }
            }
            stamina.0 = stamina.0.clamp(0., 2.)
        }
    } );
}


fn main() {
    let mut app = App::new();
    
    app
        .add_plugins((
        DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
                    title: "Bevy".into(),
                    name: Some("Bevy".into()),
                    resolution: (1920., 1080.).into(),
                    present_mode: bevy::window::PresentMode::AutoNoVsync,
                    window_theme: Some(bevy::window::WindowTheme::Dark),
                    ..default()
                }),
                ..default()
            }),
        ))
        .insert_resource(Time::<Fixed>::from_hz(48.))
        .add_plugins(PhysicsPlugins::default())
        .insert_resource(Time::<Physics>::default().with_relative_speed(4.))
        .insert_resource(SubstepCount(4))
        //.add_plugins(PhysicsDebugPlugin::default())
        .insert_resource(Gravity(Vec2::ZERO))
        .insert_resource(Handlers::default())
        .add_plugins(FoodPlugin)
        .add_plugins(SectariiPlugin)
        .add_plugins(UiPlugin)
        .add_systems(FixedUpdate, hp_system)
        .add_systems(FixedUpdate, stamina_system)
        .add_systems(FixedUpdate, satiety_system)
        .run();
}


#[derive(Resource)]
struct Handlers {
    mesh_sectarii: HashMap<usize, Handle<Mesh>>,
    material_sectarii: HashMap<usize, Handle<ColorMaterial>>,
    mesh_food: Option<Handle<Mesh>>,
    material_food: Option<Handle<ColorMaterial>>,
}
impl Default for Handlers {
    fn default() -> Self {
        Self { mesh_sectarii: HashMap::new(), material_sectarii: HashMap::new(), mesh_food: None, material_food: None }
    }
}

pub fn angle_between_2d(transform_a: &Transform, transform_b: &Transform) -> f32 {
    let dir_forward = transform_a.local_y().normalize_or_zero();
    let dir_ab = (transform_b.translation - transform_a.translation).normalize_or_zero();

    let angle_forward = atan2(dir_forward.y, dir_forward.x);
    let angle_direction = atan2(dir_ab.y, dir_ab.x);
    let angle_diff = angle_direction - angle_forward;

    let angle = angle_diff % (2. * PI);
    if angle >= PI {return angle - 2. * PI}
    else if angle < -PI {return angle + 2. * PI}
    angle
}


    //let transform_a = Transform::from_xyz(0., 0., 0.);
    //let transform_b = Transform::from_xyz(0., 10., 0.);
    //let angle = angle_between_2d(&transform_a, &transform_b);
    //println!("a:{:?}, b:{:?}, a:{}", transform_a.translation, transform_b.translation, angle);
    //let transform_a = Transform::from_xyz(0., 0., 0.);
    //let transform_b = Transform::from_xyz(10., 10., 0.);
    //let angle = angle_between_2d(&transform_a, &transform_b);
    //println!("a:{:?}, b:{:?}, a:{}", transform_a.translation, transform_b.translation, angle);
    //let transform_a = Transform::from_xyz(0., 0., 0.);
    //let transform_b = Transform::from_xyz(10., 0., 0.);
    //let angle = angle_between_2d(&transform_a, &transform_b);
    //println!("a:{:?}, b:{:?}, a:{}", transform_a.translation, transform_b.translation, angle);
    //let transform_a = Transform::from_xyz(0., 0., 0.);
    //let transform_b = Transform::from_xyz(10., -10., 0.);
    //let angle = angle_between_2d(&transform_a, &transform_b);
    //println!("a:{:?}, b:{:?}, a:{}", transform_a.translation, transform_b.translation, angle);
    //let transform_a = Transform::from_xyz(0., 0., 0.);
    //let transform_b = Transform::from_xyz(0., -10., 0.);
    //let angle = angle_between_2d(&transform_a, &transform_b);
    //println!("a:{:?}, b:{:?}, a:{}", transform_a.translation, transform_b.translation, angle);
    //let transform_a = Transform::from_xyz(0., 0., 0.);
    //let transform_b = Transform::from_xyz(-10., -10., 0.);
    //let angle = angle_between_2d(&transform_a, &transform_b);
    //println!("a:{:?}, b:{:?}, a:{}", transform_a.translation, transform_b.translation, angle);
    //let transform_a = Transform::from_xyz(0., 0., 0.);
    //let transform_b = Transform::from_xyz(-10., 0., 0.);
    //let angle = angle_between_2d(&transform_a, &transform_b);
    //println!("a:{:?}, b:{:?}, a:{}", transform_a.translation, transform_b.translation, angle);
    //let transform_a = Transform::from_xyz(0., 0., 0.);
    //let transform_b = Transform::from_xyz(-10., 10., 0.);
    //let angle = angle_between_2d(&transform_a, &transform_b);
    //println!("a:{:?}, b:{:?}, a:{}", transform_a.translation, transform_b.translation, angle);
    //let transform_a = Transform::from_xyz(0., 0., 0.);
    //let transform_b = Transform::from_xyz(0., 0., 0.);
    //let angle = angle_between_2d(&transform_a, &transform_b);
    //println!("a:{:?}, b:{:?}, a:{}", transform_a.translation, transform_b.translation, angle);
