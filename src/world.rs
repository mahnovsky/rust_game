use std::cmp::Ordering;
use std::fmt::Debug;
use std::vec::Vec;
use std::boxed::Box;
use std::any::Any;
use std::borrow::BorrowMut;
use std::cell::{Cell, RefCell, RefMut};
use std::collections::hash_map::DefaultHasher;
use std::rc::Rc;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use crate::ecs::ComponentTrait;
use crate::object_components::Lifetime;

pub trait Component {
    fn get_entity_id(&self) -> usize;
    fn reset_entity(&mut self, new_id: usize);
}

#[macro_export]
macro_rules! generate_component_impl {

    ($name: ident) => {
        impl ComponentTrait for $name {
            fn get_entity_id(&self) -> usize {
                self.entity_id
            }

            fn reset_entity(&mut self, new_id: usize) {
                self.entity_id = new_id;
            }
        }
    }
}

trait ComponentVec {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn push_none(&mut self);
    fn remove(&mut self, index: usize);
}

impl<T: 'static> ComponentVec for RefCell<Vec<Option<T>>> {
    fn as_any(&self) -> &dyn Any {
        self as &dyn Any
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self as &mut dyn Any
    }

    fn push_none(&mut self) {
        self.get_mut().push(None);
    }

    fn remove(&mut self, index: usize) {
        let v = self.get_mut();
        if index < v.len() {
            //println!("removed {}", index);
            v[index] = None;
        }
    }
}

pub struct World {
    entities: usize,
    components: HashMap<&'static str, Box<dyn ComponentVec>>,
    free_id: Vec<usize>
}

impl World {
    pub fn new() -> Self {
        Self { entities: 0, components: HashMap::new(), free_id: Vec::new() }
    }

    pub fn register_component<T: 'static>(&mut self) {
        let name = std::any::type_name::<T>();

        if !self.components.contains_key(name) {

            let mut v : Vec<Option<T>> = Vec::new();
            for _ in 0..self.entities {
                v.push(None);
            }

            self.components.insert(name, Box::new(RefCell::new(v)));
            println!("Component registered {}", name);
        }
    }

    pub fn new_entity(&mut self) -> usize {
        if let Some(x) = self.free_id.pop() {
            println!("poped {}", x);
            return x;
        }

        let entity_id = self.entities;
        for x in self.components.iter_mut() {
            x.1.push_none();
        }
        self.entities += 1;
        entity_id
    }

    pub fn print_components<T: 'static + Component + Debug>(&self) {
        let name = std::any::type_name::<T>();
            if let Some(item) = self.components.get(name) {
                if let Some(x) = item
                    .as_any()
                    .downcast_ref::<RefCell<Vec<Option<T>>>>() {
                    let component_vec = x.borrow();

                    println!("components len {}", component_vec.len());
                    for i in 0..component_vec.len() {
                        if let Some(x) = &component_vec[i] {
                            println!("Component {}: {:?}", i, x);
                        }
                        else {
                            println!("Component {} not exist", i);
                        }
                        
                    }
                }
            }
    }

    pub fn add_component<T: 'static + Component + Debug>(&mut self, entity: usize, mut component: T) {
        if entity < self.entities {
            self.register_component::<T>();
            let name = std::any::type_name::<T>();
            if let Some(item) = self.components.get_mut(name) {
                if let Some(x) = item
                    .as_any_mut()
                    .downcast_mut::<RefCell<Vec<Option<T>>>>() {
                    let component_vec  = x.get_mut();
                    component.reset_entity(entity);
                    component_vec[entity] = Some(component);
                }
            }
        }
    }

    pub fn get_component_mut<T: 'static + Component>(&mut self, entity: usize) -> Option<&mut T> {
        if entity < self.entities {
            let name = std::any::type_name::<T>();
            if let Some(item) = self.components.get_mut(name) {
                if let Some(x) = item
                    .as_any_mut()
                    .downcast_mut::<RefCell<Vec<Option<T>>>>() {
                    let components = x.get_mut();
                    if let Some(component) = components.get_mut(entity) {
                        if let Some(last) = component {
                            return Some(last);
                        }
                    }
                }
            }
        }
        None
    }

    pub fn get_component<T: 'static + Component + Clone>(&self, entity: usize) -> Option<T> {
        if entity < self.entities {
            let name = std::any::type_name::<T>();
            if let Some(item) = self.components.get(name) {
                if let Some(x) = item
                    .as_any()
                    .downcast_ref::<RefCell<Vec<Option<T>>>>() {
                    let component = x.borrow();
                    if let Some(component) = component.get(entity) {
                        if let Some(component) = component {
                            let clon = (*component).clone();
                            return Some(clon);
                        }
                    }
                }
            }
        }
        None
    }

    pub fn borrow_component_vec<T: 'static + Component>(&self) -> Option<RefMut<Vec<Option<T>>>> {
        let name = std::any::type_name::<T>();
        if let Some(cont) = self.components.get(name) {
            if let Some(x) = cont
                .as_any()
                .downcast_ref::<RefCell<Vec<Option<T>>>>() {
                return Some(x.borrow_mut());
            }
        }
        None
    }

    pub fn remove_entity(&mut self, entity_id: usize) {
        println!("Remove {}", entity_id);
        if entity_id < self.entities {
            //self.print_components::<Lifetime>();
            if self.free_id.contains(&entity_id) {
                println!( "err" );
            }

            for x in &mut self.components {
                x.1.remove(entity_id);
            }
            
            self.free_id.push(entity_id);
        }
    }
}
