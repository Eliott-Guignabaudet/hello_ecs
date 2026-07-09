mod ecs;

fn main() {
    println!("Hello, ecs!");
    let my_entity = ecs::entity::Entity::new(0, 0).id;
    println!("Hello my entity {my_entity}")
}
