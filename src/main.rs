mod ecs;
mod transform;
mod renderer;

use std::time::Instant;
use nalgebra::{Matrix4, UnitQuaternion, Vector3, Vector4};
use ecs::World;
use transform::{Position, Rotation, Scale};
use itertools::multizip;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};
use crate::renderer::{Material, Scene};
use crate::renderer::HelloRenderer;

const ENTITIES_TO_SPAWN: u32 = 20;

fn create_entities(world: &mut World) {
    for i in 0..ENTITIES_TO_SPAWN {
        let new_entity = world.spawn();
        world.add_component_to_entity(new_entity, Position { 0: Vector3::zeros() + Vector3::new(i as f32 * -1.0, i as f32 * -1.0, 0.0)});
        world.add_component_to_entity(new_entity, Rotation { 0: UnitQuaternion::identity() });
        world.add_component_to_entity(new_entity, Scale { 0: Vector3::new(1.0, 1.0, 1.0) });
        world.add_component_to_entity(new_entity, MeshRenderer {mesh_idx: 0, material_idx: 0})
    }
}
fn print_transforms(world : &World){
    let mut positions = world.borrow_component_vec_mut::<Position>().unwrap();
    let mut rotations = world.borrow_component_vec_mut::<Rotation>().unwrap();
    let mut scales = world.borrow_component_vec_mut::<Scale>().unwrap();

    let zip = multizip((positions.iter_mut(), rotations.iter_mut(), scales.iter_mut()));


    let iter = zip.filter_map(|(p, r, s)| {
        Some((p.as_mut()?, r.as_mut()?, s.as_mut()?))
    });

    for (position, rotation,scale) in iter {
        println!("Position:  {position}");
        println!("Rotation:  {rotation}");
        println!("Scale:     {scale}");
    }
}

fn rotate_objects(world: &World, delta_time: f32) {
    let mut rotations = world.borrow_component_vec_mut::<Rotation>().unwrap();

    rotations.iter_mut().for_each(|r| {
        let rotation_mut = r.as_mut().unwrap();
        rotation_mut.0 = rotation_mut.0.append_axisangle_linearized(&Vector3::new(0.0, 0.0, 90.0_f32.to_radians() * delta_time));
    });
}

fn create_render_scene(world: &World) -> Scene {
    let mut render_scene = Scene::default();
    let mut positions = world.borrow_component_vec_mut::<Position>().unwrap();
    let mut rotations = world.borrow_component_vec_mut::<Rotation>().unwrap();
    let mut scales = world.borrow_component_vec_mut::<Scale>().unwrap();
    let mut meh_renderers = world.borrow_component_vec_mut::<MeshRenderer>().unwrap();

    let zip = multizip((positions.iter_mut(), rotations.iter_mut(), scales.iter_mut(), meh_renderers.iter_mut()));
    let iter = zip.filter_map(|(p, r, s, m)| {
        Some((p.as_mut()?, r.as_mut()?, s.as_mut()?, m.as_mut()?))
    });
    for (position, rotation,_, mesh_renderer) in iter {
        let mut matrix = Matrix4::identity().append_translation(&position.0);
        matrix *= Matrix4::from(rotation.0) * Matrix4::new_scaling(1.0);

        render_scene.transforms.push(matrix);
        render_scene.model_idxs.push(mesh_renderer.mesh_idx);
        render_scene.material_idxs.push(mesh_renderer.material_idx);
        
    }
    
    render_scene
}


fn main() -> anyhow::Result<()>{
    let mut world = World::new();
    create_entities(&mut world);
    print_transforms(&world);

    let event_loop = EventLoop::new()?;
    let mut app = App::new(world);

    event_loop.run_app(&mut app)?;



    println!("Hello, ecs!");

    println!("Hello World");

    Ok(())
}

struct App {
    idx: usize,
    window_id: Option<WindowId>,
    window: Option<Window>,
    renderer: Option<HelloRenderer>,
    ecs_world: World,
    time: Instant,
    last_elapsed_time: f32,
}

impl App {
    fn new (ecs_world: World) -> Self {
        Self{
            idx: 1,
            window: None,
            window_id: None,
            renderer: None,
            ecs_world,
            time: Instant::now(),
            last_elapsed_time: 0.0,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_title("My first ECS App")
            .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0));
        let window = event_loop.create_window(window_attributes).unwrap();
        let mut renderer = HelloRenderer::new(&window).unwrap();
        
        let texture_paths : Vec<&str> = vec!["resources/T_Yoyo_Albedo.png"];
        
        let materials : Vec<Material> = vec![
               Material { base_color : Vector4::new(1.0, 1.0, 1.0, 1.0) , texture_index: Some(0)}
        ];
        renderer.load_material_resources(materials, texture_paths).unwrap();
        let correction = nalgebra::Matrix4::new_rotation(Vector3::new(90.0_f32.to_radians(), 0.0, 0.0));
        renderer.load_model_from_path("resources/yoyo.obj", correction).unwrap();
        
        
        self.window_id = Some(window.id());
        self.window = Some(window);
        self.renderer = Some(renderer);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        if event == WindowEvent::Destroyed && self.window_id == Some(window_id) {
            println!(
                "--------------------------------------------------------- Window {} Destroyed",
                self.idx
            );
            self.window_id = None;
            self.renderer = None;
            event_loop.exit();
            return;
        }

        let window = match self.window.as_mut() {
            Some(window) => window,
            None => return,
        };

        let renderer = match self.renderer.as_mut() {
            Some(render_app) => render_app,
            None => return,
        };
        

        match event {
            WindowEvent::CloseRequested => {
                println!(
                    "--------------------------------------------------------- Window {} \
                         CloseRequested",
                    self.idx
                );
                self.window = None;
            },
            WindowEvent::RedrawRequested => {
                let elapsed_time = self.time.elapsed().as_secs_f32();
                let delta_time = elapsed_time - self.last_elapsed_time;
                rotate_objects(&self.ecs_world, delta_time);
                let render_scene = create_render_scene(&self.ecs_world);
                renderer.render(window, render_scene).unwrap();
                self.last_elapsed_time = elapsed_time;
            },
            _ => (),
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }
}

#[derive(Debug)]
struct MeshRenderer {
    mesh_idx: u32,
    material_idx: u32,
}


