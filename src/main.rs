mod ecs;
mod transform;
mod renderer;

use nalgebra::{Point3, Quaternion, Vector3};
use ecs::World;
use transform::{Position, Rotation, Scale};
use itertools::multizip;


const ENTITIES_TO_SPAWN: u32 = 20;



fn main() {
    let mut world = World::new();
    create_entities(&mut world);
    print_transforms(&world);

    let _render_app = renderer::RenderApp::create();

    println!("Hello, ecs!");

    println!("Hello World")
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