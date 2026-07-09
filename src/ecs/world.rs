/// code from article:
/// https://medium.com/@jordangrilly/building-an-ecs-2-entity-management-ids-generations-and-recycling-99e289633dfb 


use std::collections::VecDeque;
use crate::ecs::entity::Entity;

pub struct World {
    next_id: u32,
    generations: Vec<u32>,
    free_ids: VecDeque<u32>,
    alive_count: usize,
}

impl World {
    pub fn new() -> Self {
        World {
            next_id: 0,
            generations: Vec::new(),
            free_ids: VecDeque::new(),
            alive_count: 0,
        }
    }
    
    pub fn spawn(&mut self) -> Entity {
        let id = if let Some(recycled) = self.free_ids.pop_front() {
            recycled
        } else { 
            let fresh = self.next_id;
            self.next_id += 1;
            fresh
        };
        
        if id as usize >= self.generations.len(){ 
            self.generations.resize(id as usize + 1, 0);
        }
        
        let generation = self.generations[id as usize];
        self.alive_count += 1;
        
        Entity::new(id, generation)
    }
    
    pub fn delete(&mut self, entity: Entity) -> bool {
        let id = entity.id as usize;
        
        if id >= self.generations.len() {
            return false;
        }
        
        if self.generations[id] != entity.generation { 
            return false;
        }
        
        self.generations[id] = self.generations[id].wrapping_add(1);
        
        self.free_ids.push_back(entity.id);
        self.alive_count -=1;
        
        true
    }
    
    #[inline]
    pub fn is_alive(&self, entity: Entity) -> bool {
        let id = entity.id as usize;
        
        id < self.generations.len() 
            && self.generations[id] == entity.generation
    }
}