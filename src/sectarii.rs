use std::time::Duration;

use bevy::prelude::*;
use avian2d::prelude::*;
use rand::Rng;

use rusty_neat::{ActFunc, NeatContinous, NN};
use crate::{angle_between_2d, food::Food, GameLayer, Handlers, Hp, Satiety, Stamina};

pub struct SectariiPlugin;
impl Plugin for SectariiPlugin {
    fn build(&self, app: &mut App) {

        let mut agent = NN::new(5, 2, Some((8, 2)), true, 0.75,
            ActFunc::SigmoidBipolar, &[ActFunc::SigmoidBipolar, ActFunc::SELU, ActFunc::Identity, ActFunc::Sinusoid, ActFunc::BinaryStep] );
        agent.set_chances(&[200, 24, 12, 16, 4, 0, 0, 12]);
        let mut neat = NeatContinous::new(&agent, 2000, 8);
        for _ in 0..20 {
            neat.speciate();
            for _ in 0..10 { neat.species_threshold_correct(); }
        }

        app
            .insert_resource(Neat(neat))
            .insert_resource(UpdateTimer(Timer::from_seconds(60.0, TimerMode::Repeating)))
            .insert_resource(StartupProcedure::default())
            .insert_resource( BioClock::new(1./2., 4.) )
            .add_systems(Startup, setup_sectarii)
            .add_systems(FixedUpdate, update_system)
            .add_systems(FixedUpdate, sensor_sight)
            .add_systems(FixedUpdate, sensor_steal)
            .add_systems(FixedUpdate, reproduction_system)
            .add_systems(FixedPostUpdate, evaluate_neat)
        ;
    }
}

#[derive(Component)]
pub struct Brain {
    pub key: usize,
    pub fitness: (f32, f32),
    pub linvel: f32,
    pub angvel: f32
}
impl Brain {
    pub fn new(key: usize) -> Self {
        Self {key, fitness: (0., 0.), linvel: 0., angvel: 0.}
    } 
}

#[derive(Component, Clone)]
pub struct Species(pub usize);

#[derive(Component, Clone)]
pub struct Sectarian;

#[derive(Component, Clone)]
struct SensorSight {
    food: (f32, f32),
    sectarian: (f32, f32),
}
impl Default for SensorSight {
    fn default() -> Self {
        Self { food: (0.,0.), sectarian: (0.,0.) }
    }
}

#[derive(Component, Clone)]
struct SensorSteal;

#[derive(Resource)]
pub struct Neat(pub NeatContinous);
// handle memory leak caused by not removing dead species handles

#[derive(Resource)]
struct UpdateTimer(Timer);
#[derive(Resource)]
struct StartupProcedure{
    p0: Timer,
    p1: Timer,
    p2: Timer,
}

impl Default for StartupProcedure {
    fn default() -> Self {
        Self { 
            p0: Timer::from_seconds(600.0, TimerMode::Once), 
            p1: Timer::from_seconds(2400.0, TimerMode::Once), 
            p2: Timer::from_seconds(720.0, TimerMode::Once), 
        }
    }
}
fn update_system(
    time: Res<Time>, 
    mut timer: ResMut<UpdateTimer>, 
    mut timer_startup: ResMut<StartupProcedure>, 
    mut neat: ResMut<Neat>,
    mut handlers: ResMut<Handlers>, 
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
){
    //let _span = info_span!("updater", name = "updater").entered();
    
    if timer.0.tick(time.delta()).just_finished() {
        let mut tbd = vec![];
        handlers.material_sectarii.iter()
            .filter(|(k, _)| neat.0.species_table.get(k).is_none() ).for_each(|(k,_)| tbd.push(*k) );
        tbd.iter().for_each(|k| {
            let h = handlers.material_sectarii.remove(k).unwrap();
            materials.remove_untracked(&h);
            let h = handlers.mesh_sectarii.remove(k).unwrap();
            meshes.remove_untracked(&h);
        } );

        if timer_startup.p2.finished() {
            if neat.0.agents.iter().next().unwrap().1.get_pruning().0 {
                neat.0.set_pruning(false, 0.2);
                timer.0.set_duration(Duration::from_secs(180));
            } else {
                neat.0.set_pruning(true, 0.2);
                timer.0.set_duration(Duration::from_secs(20));
            }
        }else{
            neat.0.set_pruning(false, 0.2);
            timer.0.set_duration(Duration::from_secs(90));
        }
    }

    if timer_startup.p0.tick(time.delta()).just_finished() {
        neat.0.add_input();
        neat.0.add_input();
        neat.0.agents.values_mut().for_each(|a| { a.sort_layers(); a.free_nodes_calc(); });
        timer_startup.p2.reset();
    }
    if timer_startup.p1.tick(time.delta()).just_finished() {
        neat.0.add_input();
        neat.0.add_input();
        neat.0.add_input();
        neat.0.add_input();
        neat.0.agents.values_mut().for_each(|a| { a.sort_layers(); a.free_nodes_calc(); });
        timer_startup.p2.reset();
    }
    timer_startup.p2.tick(time.delta());
}

fn reproduction_system(
    mut commands: Commands,
    mut neat: ResMut<Neat>,
    mut handlers: ResMut<Handlers>, 
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut entities: Query<(&mut Satiety, &Brain, &Transform), With<Sectarian>>
){
    //let _span = info_span!("reproduction", name = "reproduction").entered();
    entities.iter_mut().for_each(|(mut satiety, brain, transform)| {
        if satiety.0 > 1.75 {
            let mut rng = rand::rng();
            satiety.0 -= 1.0; // Should be > 1.

            let key = neat.0.offspring(&brain.key);
            let s = neat.0.species_assign(&key);
            handlers.mesh_sectarii.entry(s)
                .or_insert( meshes.add(Triangle2d::new(Vec2::Y * -8_f32, Vec2::X * 2_f32, Vec2::X * -2_f32)) );
            handlers.material_sectarii.entry(s)
                .or_insert( materials.add(Color::hsv( rng.random_range(0_f32..360_f32), 1_f32, 1_f32) ) );

            let mut transform = transform.clone();
            let dir = transform.local_x().normalize_or_zero();
            transform.translation += 
                Vec3::new(rng.random_range(-1_f32..=1_f32), rng.random_range(-1_f32..0_f32), 0.).normalize_or_zero() * dir * 128.;
            spawn_sectarian(&mut commands, &handlers, key, s, transform);
        }
    } );
}

fn sensor_steal(
    time: Res<Time>, 
    mut neat: ResMut<Neat>,
    col_entities: Query<(&CollidingEntities, &Parent), With<SensorSteal>>,
    mut sectarii: Query<(&mut Hp, &mut Satiety, &mut Brain), (With<Sectarian>, Without<Food>)>,
    mut foods: Query<&mut Hp, With<Food>>,
){
    //let _span = info_span!("sensor_steal", name = "sensor_steal").entered();
    col_entities.iter().for_each(|(ce, parent_key)| {
        let parent_entity = parent_key.get();

        ce.iter().for_each(|e| {
            if let Ok(mut hp) = foods.get_mut(*e) {
                let mut sectarian = sectarii.get_mut(parent_entity).unwrap();
                hp.0 -= time.delta_secs();
                sectarian.1.0 += time.delta_secs();
                sectarian.2.fitness.0 += time.delta_secs()/2.;
                neat.0.agents.get_mut(&sectarian.2.key).unwrap().fitness += time.delta_secs();
            }
            if let Ok([mut enemy, mut parent]) = sectarii.get_many_mut([*e, parent_entity ]) {
                enemy.0.0 -= time.delta_secs();
                parent.1.0 += time.delta_secs() * 0.3;
                parent.0.0 += time.delta_secs() * 0.7;
                parent.2.fitness.1 += time.delta_secs()/2.;
                neat.0.agents.get_mut(&parent.2.key).unwrap().fitness += time.delta_secs()/2.;
            }
        });
    });
}

fn sensor_sight(
    mut col_sectarii: Query<(&CollidingEntities, &Parent, &mut SensorSight, &GlobalTransform)>,
    sectarii: Query<&Transform, With<Sectarian>>,
    foods: Query<&Transform, With<Food>>,
){
    //let _span = info_span!("sensor_sight", name = "sensor_sight").entered();
    col_sectarii.par_iter_mut().for_each(|(ce, parent, mut sensor, transform)| {
        match 
            ce.iter().filter(|e| foods.get(**e).is_ok() ).map(|e| {
                let food = foods.get(*e).unwrap();
                ((food.translation - transform.translation()).length(), *e )
            } ).min_by(|a, b| a.0.partial_cmp(&b.0).unwrap() )
        {
            None => sensor.food = (0.,0.),
            Some((d, e)) => {
                let food = foods.get(e).unwrap();
                let angle = angle_between_2d(&transform.compute_transform(), food);
                sensor.food = (1. - (d / 180.), angle);
            }
        }

        match 
            ce.iter().filter(|e| **e != parent.get() && sectarii.get(**e).is_ok() ).map(|e| {
                let enemy = sectarii.get(*e).unwrap();
                ((enemy.translation - transform.translation()).length(), *e )
            } ).min_by(|a, b| a.0.partial_cmp(&b.0).unwrap() )
        {
            None => sensor.sectarian = (0.,0.),
            Some((d, e)) => {
                let enemy = sectarii.get(e).unwrap();
                let angle = angle_between_2d(&transform.compute_transform(), enemy);
                sensor.sectarian = (1. - (d / 180.), angle);
            }
        }
    });
}

#[derive(Resource)]
struct BioClock {timer_short: Timer, timer_long: Timer, state_short: isize, state_long: isize}

impl BioClock {
    fn new(timer_s: f32, timer_l: f32) -> Self {
        Self { 
            timer_short: Timer::from_seconds(timer_s, TimerMode::Repeating), 
            timer_long: Timer::from_seconds(timer_l, TimerMode::Repeating), 
            state_short: 0, 
            state_long: 1 
        }
    }
    fn tick(&mut self, delta: Duration){
        if self.timer_short.tick(delta).just_finished() {
            self.state_short = match self.state_short {
                -1 => 0,
                0 => 1,
                1 => -1,
                _ => panic!("bio_clock")
            };
        }
        if self.timer_long.tick(delta).just_finished() {
            self.state_long = match self.state_long {
                -1 => 0,
                0 => 1,
                1 => -1,
                _ => panic!("bio_clock")
            };
        }
    }
}


fn evaluate_neat(
    mut neat: ResMut<Neat>,
    mut clock: ResMut<BioClock>,
    time: Res<Time>, 
    //graph: Res<Graph>,
    sight: Query<(&Parent, &SensorSight)>,
    mut sectarii: Query<(
        &mut Brain, 
        &mut ExternalForce, &mut ExternalTorque, &Transform, 
        &LinearVelocity, &AngularVelocity, &Hp, &Satiety, &Stamina
    ), With<Sectarian>>,
) {
    clock.tick(time.delta());
    //let _span = info_span!("eval_neat", name = "eval_neat").entered();
    let inputs = sight.iter().map(|(parent_key, sensor)|{
        let parent = sectarii.get(parent_key.get()).unwrap();
        let ins = vec![ 
            sensor.food.0, sensor.food.1, sensor.sectarian.0, sensor.sectarian.1,
            parent.8.0, 
            clock.state_short as f32, clock.state_long as f32,
            parent.6.0, parent.7.0,
            parent.0.linvel, parent.0.angvel, 
        ];
        (parent.0.key, ins)
    }).collect();

    neat.0.check_integrity(&inputs);
    neat.0.forward(&inputs);

    sectarii.par_iter_mut().for_each(|(mut brain, mut force, mut torque, transform, lv, av, _, _, _)|{
        let o = neat.0.get_outputs(&brain.key);
        force.apply_force(transform.local_y().truncate().normalize_or_zero() * o[0] * 3.);
        torque.apply_torque(o[1]* 4.);
        brain.linvel = lv.length();
        brain.angvel = av.0;
    });
}


fn spawn_sectarian(commands: &mut Commands, handlers: &ResMut<Handlers>, key: usize, species: usize, transform: Transform){
    let mut e = commands.spawn(Sectarian);
    e.insert(Brain::new(key));
    e.insert(Hp(1_f32));
    e.insert(Stamina(1_f32));
    e.insert(Satiety(1_f32));
    e.insert(Species( species ));
    e.insert(transform);
    e.insert(Mesh2d( handlers.mesh_sectarii.get(&species).unwrap().clone() ));
    e.insert(MeshMaterial2d( handlers.material_sectarii.get(&species).unwrap().clone() ));
    e.insert(RigidBody::Dynamic);
    e.insert(Collider::triangle_unchecked(Vec2::Y * -10_f32, Vec2::X * 2.5_f32, Vec2::X * -2.5_f32));
    e.insert(CollisionLayers::new([GameLayer::Default, GameLayer::Sectarii], 
        [GameLayer::Default, GameLayer::Sectarii, GameLayer::Foods, GameLayer::Bullets]));
    e.insert(ColliderDensity(0.001));
    e.insert(Friction::new(0.4));
    e.insert(LinearDamping(2.0));
    e.insert(AngularDamping(1.0));
    e.insert(ExternalForce::ZERO.with_persistence(false));
    e.insert(ExternalTorque::ZERO.with_persistence(false));
    e.insert(LinearVelocity::ZERO);
    e.insert(AngularVelocity::ZERO);

    e.with_child(( 
        SensorSight::default(),
        Collider::compound(vec![
            (
                Position::default(), Rotation::default(),
                Collider::triangle_unchecked(Vec2::new(0., 0.), Vec2::new(1.5, 3.) * 40., Vec2::new(-1.5, 3.) * 40.), 
            ),
            (
                Position::default(), Rotation::default(),
                Collider::circle(32.), 
            )
        ]),
        Sensor,
        CollisionLayers::new([GameLayer::Foods, GameLayer::Sectarii], [GameLayer::Foods, GameLayer::Sectarii]),
        CollidingEntities::default(),
        GlobalTransform::default()
    ));

    e.with_child(( 
        SensorSteal,
        Collider::circle(2.5_f32),
        Sensor,
        CollisionLayers::new([GameLayer::Foods, GameLayer::Sectarii], [GameLayer::Foods, GameLayer::Sectarii]),
        CollidingEntities::default(),
    ));
}

fn setup_sectarii(
    mut commands: Commands,
    mut neat: ResMut<Neat>,
    mut handlers: ResMut<Handlers>, 
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut rng = rand::rng();
    neat.0.speciate();
    neat.0.speciate();
    neat.0.speciate();
    neat.0.speciate();
    
    neat.0.species_table.keys().for_each(|k| {
        handlers.mesh_sectarii.insert(*k, meshes.add(Triangle2d::new(Vec2::Y * -8_f32, Vec2::X * 2_f32, Vec2::X * -2_f32)) );
        handlers.material_sectarii.insert(*k, materials.add(Color::hsv( rng.random_range(0_f32..360_f32), 1_f32, 1_f32)) );
    } );
    
    neat.0.agents.iter().for_each(|(k, a)| {
        spawn_sectarian(&mut commands, &handlers, *k, a.species,
            Transform::from_xyz(rng.random_range(-6_000_f32..6_000_f32), rng.random_range(-6_000_f32..6_000_f32), 0.));
    });
}
