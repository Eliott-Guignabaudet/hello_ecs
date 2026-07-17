mod ecs;
mod transform;
mod renderer;


use nalgebra::{Point3, Quaternion, Vector3};
use ecs::World;
use transform::{Position, Rotation, Scale};
use itertools::multizip;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};
use crate::renderer::HelloRenderer;

const ENTITIES_TO_SPAWN: u32 = 20;
#[derive(Clone, Debug, Copy)]
struct Vertex {
    pos: [f32; 4],
    color: [f32; 4],
}

fn create_entities(world: &mut World) {
    for _ in 0..ENTITIES_TO_SPAWN {
        let new_entity = world.spawn();
        world.add_component_to_entity(new_entity, Position { 0: Point3::origin() });
        world.add_component_to_entity(new_entity, Rotation { 0: Quaternion::identity() });
        world.add_component_to_entity(new_entity, Scale { 0: Vector3::new(1.0, 1.0, 1.0) });
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


fn main() -> anyhow::Result<()>{
    let mut world = World::new();
    create_entities(&mut world);
    print_transforms(&world);

    let event_loop = EventLoop::new()?;
    let mut app = App::new();

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
}

impl App {
    fn new () -> Self {
        Self{
            idx: 1,
            window: None,
            window_id: None,
            renderer: None,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_title("My first ECS App")
            .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0));
        let window = event_loop.create_window(window_attributes).unwrap();
        let renderer = HelloRenderer::new(&window).unwrap();
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
                renderer.render(window).unwrap();
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


