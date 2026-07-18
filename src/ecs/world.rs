use std::cell::{RefCell, RefMut};
/// code from article:
/// https://medium.com/@jordangrilly/building-an-ecs-2-entity-management-ids-generations-and-recycling-99e289633dfb 


use std::collections::VecDeque;
use crate::ecs::component::ComponentVec;
use crate::ecs::entity::Entity;

pub struct World {
    next_id: u32,
    generations: Vec<u32>,
    free_ids: VecDeque<u32>,
    alive_count: usize,
    component_vecs: Vec<Box<dyn ComponentVec>>,
}

impl World {
    pub fn new() -> Self {
        World {
            next_id: 0,
            generations: Vec::new(),
            free_ids: VecDeque::new(),
            alive_count: 0,
            component_vecs: Vec::new(),
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
            for component_vec in self.component_vecs.iter_mut() {
                component_vec.push_none();
            }
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

        // When entity is deleted, all it's components are set to none
        self.component_vecs.iter_mut().for_each(|component_vec| {
           component_vec.set_none(id);
        });
        
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


    pub fn add_component_to_entity<ComponentType: 'static>(
        &mut self,
        entity: Entity,
        component: ComponentType,
    ) {
        if !self.is_alive(entity) { 
            return;
        }
        // Search for any existing ComponentVecs that match the type of the component being added.
        for component_vec in self.component_vecs.iter_mut() {
            if let Some(component_vec) = component_vec
                .as_any_mut()
                .downcast_mut::<RefCell<Vec<Option<ComponentType>>>>()
            {
                component_vec.borrow_mut()[entity.id as usize] = Some(component);
                return;
            }
        }

        // No matching component storage exists yet, so we have to make one.
        let mut new_component_vec: Vec<Option<ComponentType>> =
            Vec::with_capacity(self.generations.len());

        // All existing entities don't have this component, so we give them `None`
        for _ in 0..self.generations.len() {
            new_component_vec.push(None);
        }

        // Give this Entity the Component.
        new_component_vec[entity.id as usize] = Some(component);
        self.component_vecs
            .push(Box::new(RefCell::new(new_component_vec)));
    }


    pub fn borrow_component_vec_mut<ComponentType: 'static>(
        &self,
    ) -> Option<RefMut<'_, Vec<Option<ComponentType>>>> {
        for component_vec in self.component_vecs.iter() {
            if let Some(component_vec) = component_vec
                .as_any()
                .downcast_ref::<RefCell<Vec<Option<ComponentType>>>>()
            {
                return Some(component_vec.borrow_mut());
            }
        }
        None
    }
}