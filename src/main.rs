mod ecs;
mod transform;
mod renderer;
mod camera_movements;

use std::time::Instant;
use nalgebra::{Matrix4, UnitQuaternion, Vector3, Vector4};
use ecs::World;
use transform::{Position, Rotation, Scale};
use itertools::{multizip};
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Window, WindowId};
use rand::{random_range};
use crate::camera_movements::{get_camera_movement, MovementFlags};
use crate::renderer::{CameraData, DirectionalLight, Material, Scene};
use crate::renderer::HelloRenderer;

const ENTITIES_TO_SPAWN: u32 = 336;
const LIGHT_POS: Vector3<f32> = Vector3::new(0.0, 20.0, 10.0 );

fn create_entities(world: &mut World) {
    
    // Create Frogs
    let space_frogs = 2.0;
    for i in 0..ENTITIES_TO_SPAWN {
        for j in 0..ENTITIES_TO_SPAWN {
            let new_entity = world.spawn();
            world.add_component_to_entity(new_entity, Position { 0: Vector3::zeros() + Vector3::new(i as f32 * -space_frogs , j as f32 * -space_frogs, 0.0) });
            world.add_component_to_entity(new_entity, Rotation { 0: UnitQuaternion::identity() });
            world.add_component_to_entity(new_entity, Scale { 0: Vector3::new(1.0, 1.0, 1.0) });
            world.add_component_to_entity(new_entity, MeshData { mesh_idx: 0, material_idx: 0 });
            let x_angular = random_range(-1.0..1.0);
            let y_angular = random_range(-1.0..1.0);
            let z_angular = random_range(-1.0..1.0);
            let angular_speed = random_range(45.0..90.0);

            world.add_component_to_entity(new_entity, AngularVelocity { velocity: Vector3::new(x_angular, y_angular, z_angular).normalize(), speed: angular_speed});
            
        }
        
    }
    
    // Create Light
    let light_object = world.spawn();
    world.add_component_to_entity(light_object, Position { 0: LIGHT_POS });
    world.add_component_to_entity(light_object, Rotation { 0: UnitQuaternion::identity() });
    world.add_component_to_entity(light_object, Scale { 0: Vector3::new(1.0, 1.0, 1.0) });
    //world.add_component_to_entity(light_object, MeshData {mesh_idx: 1, material_idx: 1});
    
    // Create Camera
    let camera = world.spawn();
    let cam_data = create_camera();
    world.add_component_to_entity(camera, Position { 0: cam_data.position });
    world.add_component_to_entity(camera, Rotation { 0: cam_data.rotation });
    world.add_component_to_entity(camera, Camera{});
    world.add_component_to_entity(camera, CameraMovementInput {0: MovementFlags::empty()});
    world.add_component_to_entity(camera, CameraRotationInput {0: (0.0, 0.0)});
    
}

fn rotate_objects(world: &World, delta_time: f32) {
    let mut rotations = world.borrow_component_vec_mut::<Rotation>().unwrap();
    let angular_velocity = world.borrow_component_vec::<AngularVelocity>().unwrap();
    let zip = rotations.iter_mut().zip(angular_velocity.iter());
    let iter = zip.filter_map(|(r, av)| {
        Some((r.as_mut()?, av.as_ref()?))
    });

    for (rotation, angular_velocity) in iter {
        rotation.0 *= UnitQuaternion::new((angular_velocity.velocity * delta_time * angular_velocity.speed.to_radians()));
    }
}

fn create_render_scene(world: &World) -> Scene {
    let mut render_scene = Scene::default();
    let positions = world.borrow_component_vec::<Position>().unwrap();
    let rotations = world.borrow_component_vec::<Rotation>().unwrap();
    let scales = world.borrow_component_vec::<Scale>().unwrap();
    let meh_renderers = world.borrow_component_vec::<MeshData>().unwrap();
    let cameras = world.borrow_component_vec::<Camera>().unwrap();

    let zip = multizip((positions.iter(), rotations.iter(), scales.iter(), meh_renderers.iter()));
    let iter = zip.filter_map(|(p, r, s, m)| {
        Some((p.as_ref()?, r.as_ref()?, s.as_ref()?, m.as_ref()?))
    });
    for (position, rotation, scale, mesh_renderer) in iter {
        let mut matrix = Matrix4::identity().append_translation(&position.0);
        matrix *= Matrix4::from(rotation.0) * Matrix4::new_nonuniform_scaling(&scale.0) ;

        render_scene.transforms.push(matrix);
        render_scene.model_idxs.push(mesh_renderer.mesh_idx);
        render_scene.material_idxs.push(mesh_renderer.material_idx);
        
    }
    let zip = multizip((positions.iter(), rotations.iter(), cameras.iter()));
    let iter = zip.filter_map(|(p, r, c)| {
        Some((p.as_ref()?, r.as_ref()?, c.as_ref()?))
    });
    for (position, rotation, _) in iter {
        render_scene.camera_data = CameraData{ position: position.0, rotation: rotation.0};
    }
    
    
    render_scene.directional_light = DirectionalLight {
        position: LIGHT_POS
    };
    
    
    render_scene
}

fn create_camera() -> CameraData {
    let rotation =
        UnitQuaternion::from_axis_angle(&Vector3::x_axis(), 125.0_f32.to_radians())
            *   UnitQuaternion::from_axis_angle(&Vector3::y_axis(), 180.0_f32.to_radians())
        ;
    let position = Vector3::new(0.0, -5.0 ,-10.0);
    CameraData { position, rotation }
}


fn update_camera_movements_inputs(world: &World, movement_flags: MovementFlags){
    let mut camera_movement = world.borrow_component_vec_mut::<CameraMovementInput>().unwrap();
    camera_movement
        .iter_mut()
        .filter_map(|c| Some(c.as_mut()?))
        .for_each(|c| c.0 = movement_flags)
}
fn update_camera_rotation_inputs(world: &World, mouse_input_delta: (f64, f64)){
    let mut camera_rotation = world.borrow_component_vec_mut::<CameraRotationInput>().unwrap();
    camera_rotation
        .iter_mut()
        .filter_map(|c| Some(c.as_mut()?))
        .for_each(|c| c.0 = mouse_input_delta)
}

fn rotate_cameras(world: &World){
    let rotation_speed = 0.1;

    let mut camera_rotation = world.borrow_component_vec_mut::<CameraRotationInput>().unwrap();
    let mut rotations = world.borrow_component_vec_mut::<Rotation>().unwrap();
    
    camera_rotation
        .iter_mut()
        .zip(rotations.iter_mut())
        .filter_map(|(c, r)| Some((c.as_mut()?, r.as_mut()?)))
        .for_each(|(c, r)| {
            r.0 =  UnitQuaternion::from_axis_angle(&Vector3::z_axis(), (-c.0.0 as f32 * rotation_speed).to_radians())
                * r.0
                * UnitQuaternion::from_axis_angle(&Vector3::x_axis(), (-c.0.1 as f32 * rotation_speed).to_radians());
        });
    
}

fn move_cameras(world: &World, delta_time: f32) {
    let cam_speed = 10.0;
    let mut camera_movements = world.borrow_component_vec_mut::<CameraMovementInput>().unwrap();
    let mut rotations = world.borrow_component_vec_mut::<Rotation>().unwrap();
    let mut positions = world.borrow_component_vec_mut::<Position>().unwrap();
    
    multizip(
        (camera_movements.iter_mut(), 
         positions.iter_mut(), 
         rotations.iter_mut()))
        .filter_map(|(c, p, r)| Some((c.as_mut()?, p.as_mut()?, r.as_mut()?)))
        .for_each(|(c, p, r)| {
            let mut movement_vector = get_camera_movement(c.0);
            movement_vector = r.0.transform_vector(&movement_vector);
            p.0 = p.0 + movement_vector * delta_time * cam_speed;
        });
}


struct App {
    idx: usize,
    window_id: Option<WindowId>,
    window: Option<Window>,
    renderer: Option<HelloRenderer>,
    ecs_world: World,
    time: Instant,
    last_elapsed_time: f32,
    mouse_delta: (f64, f64),
    camera_movement_flag: MovementFlags,
    is_focused: bool,
    can_focus: bool,
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
            mouse_delta: (0.0, 0.0),
            camera_movement_flag: MovementFlags::empty(),
            is_focused: false,
            can_focus: false,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_title("My first ECS App")
            .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0));
        let window = event_loop.create_window(window_attributes).unwrap();
        window.set_cursor_grab(CursorGrabMode::Confined).unwrap();
        window.set_cursor_visible(false);
        self.is_focused = true;
        self.can_focus = false;
        let mut renderer = HelloRenderer::new(&window).unwrap();
        
        let texture_paths : Vec<&str> = vec!["resources/T_Yoyo_Albedo.png", "resources/empty.png"];
        
        let materials : Vec<Material> = vec![
               Material { base_color : Vector4::new(1.0, 1.0, 1.0, 1.0) , texture_index: Some(0)},
               Material { base_color : Vector4::new(1.0, 1.0, 1.0, 1.0) , texture_index: Some(1)},
        ];
        renderer.load_material_resources(materials, texture_paths).unwrap();
        let correction = Matrix4::new_rotation(Vector3::new(90.0_f32.to_radians(), 0.0, 0.0));
        renderer.load_model_from_path("resources/yoyo.obj", correction).unwrap();
        renderer.load_model_from_path("resources/cube.obj", correction).unwrap();
        
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
                
                // input systems
                update_camera_movements_inputs(&self.ecs_world, self.camera_movement_flag);
                update_camera_rotation_inputs(&self.ecs_world, self.mouse_delta);
                
                // update systems
                rotate_objects(&self.ecs_world, delta_time);
                rotate_cameras(&self.ecs_world);
                move_cameras(&self.ecs_world, delta_time);
                
                // draw systems
                let render_scene = create_render_scene(&self.ecs_world);
                renderer.render(window, render_scene).unwrap();
                self.last_elapsed_time = elapsed_time;
                self.mouse_delta = (0.0, 0.0);
            },
            WindowEvent::Resized(_) => {
                renderer.recreate_swapchain(window).unwrap();
                
            },
            WindowEvent::MouseInput {state, button, ..} => {
                if state  == ElementState::Pressed && button == MouseButton::Left && !self.is_focused && self.can_focus {
                    window.set_cursor_grab(CursorGrabMode::Confined).unwrap();
                    window.set_cursor_visible(false);
                    self.is_focused = true;
                }
            }
            WindowEvent::CursorEntered {..} => {
                self.can_focus = true;
            }
            WindowEvent::CursorLeft {..} => {
                self.can_focus = false;
            }
            WindowEvent::KeyboardInput {event, ..} => {
                if event.state == ElementState::Pressed && self.is_focused { 
                    match event.physical_key {
                        PhysicalKey::Code(KeyCode::Escape) => {
                            window.set_cursor_grab(CursorGrabMode::None).unwrap();
                            window.set_cursor_visible(true);
                            self.is_focused = false;
                        },
                        _ => {}
                    }
                }
            }
            _ => (),
        }
    }

    fn device_event(&mut self, _: &ActiveEventLoop, _: DeviceId, event: DeviceEvent) {
        match event {
            DeviceEvent::MouseMotion { delta} => {
                if self.is_focused { 
                    self.mouse_delta = (self.mouse_delta.0 +delta.0, self.mouse_delta.1 +delta.1);
                }
            }
            DeviceEvent::Key(event) => {
                if event.state == ElementState::Pressed && self.is_focused {
                    match event.physical_key {
                        PhysicalKey::Code(KeyCode::KeyW) => self.camera_movement_flag.insert(MovementFlags::FORWARD),
                        PhysicalKey::Code(KeyCode::KeyS) => self.camera_movement_flag.insert(MovementFlags::BACKWARD),
                        PhysicalKey::Code(KeyCode::KeyA) => self.camera_movement_flag.insert(MovementFlags::LEFT),
                        PhysicalKey::Code(KeyCode::KeyD) => self.camera_movement_flag.insert(MovementFlags::RIGHT),
                        _ => { }
                    }
                }
                else if event.state == ElementState::Released && self.is_focused {
                    match event.physical_key {
                        PhysicalKey::Code(KeyCode::KeyW) => self.camera_movement_flag.remove(MovementFlags::FORWARD),
                        PhysicalKey::Code(KeyCode::KeyS) => self.camera_movement_flag.remove(MovementFlags::BACKWARD),
                        PhysicalKey::Code(KeyCode::KeyA) => self.camera_movement_flag.remove(MovementFlags::LEFT),
                        PhysicalKey::Code(KeyCode::KeyD) => self.camera_movement_flag.remove(MovementFlags::RIGHT),
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }
    
}

#[derive(Debug)]
struct MeshData {
    mesh_idx: u32,
    material_idx: u32,
}

struct AngularVelocity{
    velocity: Vector3<f32>,
    speed: f32,
}

struct Camera;
struct CameraMovementInput(MovementFlags);
struct CameraRotationInput((f64, f64));



fn main() -> anyhow::Result<()>{
    println!("Hello, ecs!");
    let mut world = World::new();
    create_entities(&mut world);
    let event_loop = EventLoop::new()?;
    let mut app = App::new(world);

    event_loop.run_app(&mut app)?;
    
    Ok(())
}

