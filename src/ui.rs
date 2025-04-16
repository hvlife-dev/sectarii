use std::fs;

use bevy::{input::common_conditions::input_just_pressed, prelude::*, render::camera::Viewport};
use bevy_egui::{EguiContexts, EguiPlugin, egui};
use bevy_pancam::{PanCam, PanCamPlugin};
use iyes_perf_ui::prelude::*;
use rusty_neat::visu;

use crate::{sectarii::{Brain, Neat, Sectarian, Species}, Hp, Satiety, Stamina};


pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
            .add_plugins(bevy::diagnostic::EntityCountDiagnosticsPlugin)
            .add_plugins(bevy::render::diagnostic::RenderDiagnosticsPlugin)
            .add_plugins(PerfUiPlugin::default())
            .add_plugins(PanCamPlugin)
            .add_plugins(EguiPlugin)
            .insert_resource(Graph::default())
            //.add_systems(Startup, minimap_camera)
            .add_systems(Startup, main_camera)
            .add_systems(Update, (save_load, ui_update).chain())
            .add_systems(Update, 
                cursor_system.run_if(input_just_pressed(MouseButton::Right)))
        ;
    }
}

fn save_load(
    mut neat: ResMut<Neat>,
    mut graph: ResMut<Graph>,
    mut contexts: EguiContexts,
    keys: Res<ButtonInput<KeyCode>>,
){
    if keys.just_pressed(KeyCode::Digit1) {
        if let Some(a) = neat.0.agents.get(&graph.key) {
            a.save("assets/saved.toml");
            visu(a, Some("assets/saved.svg"));
        }
    }
    if keys.just_pressed(KeyCode::Digit2) {
        if let Some(a) = neat.0.agents.get_mut(&graph.key) {
            let ins = a.size.0;
            a.load("assets/saved.toml");
            while a.size.0 < ins { a.add_input(); }
            a.sort_layers(); a.free_nodes_calc();

            graph.key_old = graph.key + 1; // force regenerate
            contexts.ctx_mut().forget_all_images();
        }
    }
}

fn ui_update(
    time: Res<Time>, 
    mut contexts: EguiContexts,
    neat: Res<Neat>,
    mut graph: ResMut<Graph>,
    sectarii: Query<(&Transform, &Brain, &Species, &Hp, &Satiety, &Stamina), (With<Sectarian>, Without<Halo>)>,
    mut halo: Query<&mut Transform, (With<Halo>, Without<Sectarian>)>
) {
    let mut hp = 0.;
    let mut satiety = 0.;
    let mut stamina = 0.;
    let mut velocity = 0.;
    let mut angle = 0.;
    let mut species = 0;
    let mut sectarian_key = 0;
    if graph.entity.is_some() {
        if let Ok(sectarian) = sectarii.get(graph.entity.unwrap()) {
            let mut halo_t = halo.get_single_mut().unwrap();
            halo_t.translation = sectarian.0.translation;

            sectarian_key = sectarian.1.key;
            species = sectarian.2.0;
            hp = sectarian.3.0;
            satiety = sectarian.4.0;
            stamina = sectarian.5.0;
            velocity = sectarian.1.linvel;
            angle = sectarian.1.angvel;
        }
    }
    if graph.key != graph.key_old {
        graph.key_old = graph.key;
        let path = "assets/temp/output".to_string() + &graph.key.to_string() + ".svg";
        visu(neat.0.agents.get(&graph.key).unwrap(), Some(&path));
    }
    egui::Window::new("Sectarii").show(contexts.ctx_mut(), |ui| {
        ui.label(format!("Time: {:>.0}", time.elapsed_secs()));
        ui.label(format!("Population size: {}", neat.0.agents.len()));
        ui.label(format!("Species amount: {}", neat.0.species_table.len() ));
        ui.label(format!("Species threshold: {:>.1}", neat.0.species_threshold ));
        ui.separator();
        ui.label(format!("Key: {}", sectarian_key));
        ui.label(format!("Species: {}", species));
        ui.separator();
        ui.add(egui::ProgressBar::new(velocity / 50.).fill(egui::Color32::from_rgb(48, 48, 8))
            .text(format!("Velocity linear:  {:>.1} p/s", velocity)));
        ui.add(egui::ProgressBar::new(angle.abs() / 4.).fill(egui::Color32::from_rgb(48, 48, 8))
            .text(format!("Velocity angular: {:>.1} rad/s", angle)));
        ui.separator();
        ui.add(egui::ProgressBar::new(hp/2.).fill(egui::Color32::from_rgb(64, 0, 0)).text("Hp"));
        ui.add(egui::ProgressBar::new(satiety/2.).fill(egui::Color32::from_rgb(0, 64, 0)).text("Satiety"));
        ui.add(egui::ProgressBar::new(stamina/2.).fill(egui::Color32::from_rgb(0, 0, 64)).text("Stamina"));
        ui.separator();
        ui.add( egui::Image::new(
            &("file://assets/temp/output".to_string() + &graph.key.to_string() + ".svg")
        ).max_size(egui::Vec2::new(300., 600.)).fit_to_original_size(2.) );
    });
}

#[derive(Resource)]
pub struct Graph {
    pub key: usize,
    key_old: usize,
    entity: Option<Entity>,
}
impl Default for Graph {
    fn default() -> Self {
        Self { key: 0, key_old: 1, entity: None }
    }
}
#[derive(Component)]
pub struct Halo;

fn cursor_system(
    q_window: Query<&Window>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    sectarii: Query<(&Brain, &Transform, Entity), With<Sectarian>>,
    mut graph: ResMut<Graph>
) {
    let (camera, camera_transform) = q_camera.single();
    let window = q_window.single();
    if let Some(world_position) = window.cursor_position()
        .and_then(|cursor| Some( camera.viewport_to_world(camera_transform, cursor) ) )
        .map(|ray| ray.unwrap().origin.truncate())
    {
        let closest = sectarii.iter()
            .map(|(b,t,e)| (b.key, t.translation.truncate().distance(world_position), e ) )
            .min_by(|a,b| a.1.partial_cmp(&b.1).unwrap() );
        if let Some((k,_,e)) = closest {graph.key = k; graph.entity = Some(e);}
    }
}


fn main_camera(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((Camera2d::default(), PanCam::default(), MainCamera));
    commands.spawn(PerfUiEntryFPS::default());
    commands.spawn((
        Halo,
        Transform::from_xyz(0., 0., 0.),
        Mesh2d(meshes.add( Annulus::new(12., 18.) )),
        MeshMaterial2d(materials.add( Color::hsva(60., 0.75, 0.75, 0.1) ))
    ));
    egui_extras::install_image_loaders(contexts.ctx_mut());
    fs::create_dir_all("assets/temp").unwrap();
}

#[allow(dead_code)]
fn minimap_camera(mut commands: Commands) {
    let mut o_proj = OrthographicProjection::default_2d();
    o_proj.scaling_mode = 
        bevy::render::camera::ScalingMode::FixedVertical { viewport_height: 3000. };

    commands.spawn((
        Camera2d::default(), 
        Camera {
            order: 2,
            viewport: Some(Viewport {
                physical_position: UVec2::new(5, 775),
                physical_size: UVec2::new(300, 300),
                ..Default::default()
            }),
            ..Default::default()
        },
        o_proj,
        MinimapCamera
    ));
}


#[derive(Component)]
struct MainCamera;
#[derive(Component)]#[allow(unused_variables)]
struct MinimapCamera;
