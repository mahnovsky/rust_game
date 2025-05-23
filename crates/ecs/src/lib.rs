use bit_vec::BitVec;
use std::any::Any;
use std::cell::{Cell, RefCell};
use std::clone::Clone;
use std::collections::hash_map::DefaultHasher;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::rc::{Rc, Weak};

mod events;

pub use events::*;

#[derive(Clone, Copy, Debug)]
pub enum EcsEvent {
    EntityCreated(EntityId),
    EntityDestroyed(EntityId),
}

pub trait Component {
    const INDEX: usize;

    fn get_entity_id(&self) -> Option<EntityId>;
}

trait ComponentContainer {
    fn as_any(&self) -> &dyn Any;
    fn reset(&mut self, index: usize);
}

type ComponentContainerVec<T> = Rc<RefCell<Vec<Option<T>>>>;

impl<T: 'static + Component> ComponentContainer for ComponentContainerVec<T> {
    

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn reset(&mut self, id: usize) {
        let mut s = self.deref().borrow_mut();
        if id < s.len() {
            s[id] = None;
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Hash, Eq)]
pub struct EntityId(usize);

//#[derive(Debug)]
pub struct Entity {
    weak_ecs: EcsWeak,
    entity_id: EntityId,
    events: RefCell<EventSystem>,
}

impl PartialEq for Entity {
    fn eq(&self, other: &Self) -> bool {
        self.entity_id == other.entity_id
    }
}

impl Eq for Entity {
    fn assert_receiver_is_total_eq(&self) {}
}

impl Hash for Entity {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.entity_id.hash(state);
    }
}

impl Entity {
    pub fn new(ecs: &EcsRc) -> Weak<Self> {
        let id = {
            let ecs = ecs.deref().borrow();
            ecs.create_entity_handle()
        };
        let res = Rc::new(Self {
            weak_ecs: Rc::downgrade(ecs),
            entity_id: id,
            events: RefCell::new(EventSystem::new()),
        });

        let ecs = ecs.deref().borrow();

        ecs.add_entity(res)
    }

    pub fn get_id(&self) -> EntityId {
        self.entity_id
    }

    pub fn kill(&self) {
        if let Some(rc_ecs) = self.weak_ecs.upgrade() {
            let ecs = rc_ecs.borrow();

            ecs.remove_entity(self.entity_id);
        }
    }

    pub fn add_component<T: 'static + Component>(&self, creator: impl FnOnce() -> T) {
        if let Some(ecs) = self.weak_ecs.upgrade() {
            let mut ecs = ecs.deref().borrow_mut();

            ecs.add_component(self, creator);
        }
    }

    pub fn visit<T: 'static + Component>(&self, f: impl FnOnce(&mut Option<T>)) {
        if let Some(ecs) = self.weak_ecs.upgrade() {
            let ecs = ecs.deref().borrow();
            if ecs.is_componet_exist::<T>() {
                ecs.visit(self.entity_id, f);
            }
        }
    }

    pub fn get_component_clone<T: 'static + Component + Clone>(&self) -> Option<T> {
        let ecs = self.weak_ecs.upgrade()?;
        let ecs = ecs.deref().borrow();

        if let Some(container) = ecs.get_container::<T>() {
            let container = container.deref().borrow();

            if let Some(component) = container.get(self.entity_id.0) {
                return component.clone();
            }
        }

        None
    }

    pub fn push_event<E: Sized + Clone + 'static>(&self, event: E) {
        let mut events = self.events.borrow_mut();

        events.push_event(event);
    }

    pub fn process_event<E: Sized + Clone + 'static, T: 'static + Component + Listener<E>>(&self) {
        
        let events = self.events.borrow_mut();
        self.visit::<T>(|component|{
            if let Some(component) = component {
                events.process_event(component);
            }
        });
    }

    pub fn clean_events<E: Sized + Clone + 'static>(&self) {
        let mut events = self.events.borrow_mut();

        events.clear::<E>();
    } 
}

struct EntityCash {
    entities: Vec<Option<Rc<Entity>>>,
    free_indexes: Vec<EntityId>,
    check_bit: BitVec,
}

impl EntityCash {
    const GROW_SIZE: usize = 128;
    fn new() -> Self {
        Self {
            entities: Vec::new(),
            free_indexes: Vec::with_capacity(Self::GROW_SIZE),
            check_bit: BitVec::from_elem(Self::GROW_SIZE, false),
        }
    }

    fn create_entity_handle(&mut self) -> EntityId {
        if let Some(id) = self.free_indexes.pop() {
            self.check_bit.set(id.0, true);

            return id;
        }

        let index = self.entities.len();
        self.entities.resize_with(index + 1, || None);
        let iter = (index + Self::GROW_SIZE)..index;
        self.free_indexes.extend(iter.map(EntityId));
        if self.check_bit.len() <= index {
            self.check_bit
                .grow(self.check_bit.len() + Self::GROW_SIZE, false);
        }
        self.check_bit.set(index, true);

        EntityId(index)
    }

    fn add_entity(&mut self, entity: Rc<Entity>) -> Weak<Entity> {
        self.entities[entity.get_id().0] = Some(entity.clone());

        Rc::downgrade(&entity)
    }

    fn remove_entity(&mut self, entity_id: EntityId) {
        self.free_indexes.push(entity_id);
        self.entities[entity_id.0] = None;
        self.check_bit.set(entity_id.0, false);
    }

    fn is_entity_alive(&self, id: EntityId) -> bool {
        self.check_bit.get(id.0).unwrap()
    }

    fn get_alive_check(&self) -> BitVec {
        self.check_bit.clone()
    }
}

pub struct Ecs {
    entity_counter: Cell<usize>,
    components: Vec<Option<Box<dyn ComponentContainer>>>,
    entity_cache: RefCell<EntityCash>,
    pub events: RefCell<EventSystem>,
}

impl Debug for Ecs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Ecs etities count {}", self.entity_counter.get())
    }
}

impl Default for Ecs {
    fn default() -> Self {
        Self {
            entity_counter: Cell::new(0),
            components: Vec::new(),
            entity_cache: RefCell::new(EntityCash::new()),
            events: RefCell::new(EventSystem::new()),
        }
    }
}

pub type EcsRc = Rc<RefCell<Ecs>>;
pub type EcsWeak = Weak<RefCell<Ecs>>;
pub type EntityWeak = Weak<Entity>;

impl Ecs {
    pub fn new() -> Self {
        let mut v = Vec::<Option<Box<dyn ComponentContainer>>>::with_capacity(128);
        v.resize_with(128, || None);
        Self {
            components: v,
            entity_counter: Cell::new(0),
            entity_cache: RefCell::new(EntityCash::new()),
            events: RefCell::new(EventSystem::new()),
        }
    }

    fn create_entity_handle(&self) -> EntityId {
        let mut cache = self.entity_cache.borrow_mut();

        cache.create_entity_handle()
    }

    fn add_entity(&self, entity: Rc<Entity>) -> EntityWeak {
        let mut cache = self.entity_cache.borrow_mut();

        let mut events = self.events.borrow_mut();
        events.push_event(EcsEvent::EntityCreated(entity.entity_id));
        drop(events);

        cache.add_entity(entity)
    }

    pub fn remove_entity(&self, entity_id: EntityId) {
        let mut cache = self.entity_cache.borrow_mut();

        let mut events = self.events.borrow_mut();
        events.push_event(EcsEvent::EntityDestroyed(entity_id));
        drop(events);

        println!("Entity {:?} removed", entity_id);

        cache.remove_entity(entity_id);
    }

    pub fn get_entity(&self, id: EntityId) -> Option<Rc<Entity>> {
        let cache = self.entity_cache.borrow_mut();
        if cache.is_entity_alive(id) {
            return cache.entities[id.0].clone().into();
        }
        None
    }

    pub fn process_events<E: Sized + Clone + 'static, T: 'static + Component + Listener<E>>(&self) {
        let cache = self.entity_cache.borrow();
        for entity in cache.entities.iter() {
            if let Some(entity) = entity { 
                entity.process_event::<E, T>();
            }
        }
    }

    pub fn process_events_all<E: Sized + Clone + 'static>(&self) {
        let cache = self.entity_cache.borrow();
        for id in 0..cache.entities.len() {
            if cache.is_entity_alive(EntityId(id)) {
                
            }
        }
    }

    pub fn clean_events<E: Sized + Clone + 'static>(&self) {
        let cache = self.entity_cache.borrow();
        for entity in cache.entities.iter() {
            if let Some(entity) = entity { 
                entity.clean_events::<E>();
            }
        }
    } 

    pub fn add_component<T>(&mut self, entity: &Entity, creator: impl FnOnce() -> T)
    where
        T: 'static + Component,
    {
        let entity_id = entity.entity_id.0;
        if let Some(cont) = self.get_container::<T>() {
            let mut c1 = cont.deref().borrow_mut();
            if c1.len() <= entity_id {
                c1.resize_with(entity_id + 1, || None);
            }
            c1[entity_id] = Some(creator());
        } else {
            let mut v = Vec::<Option<T>>::new();
            v.resize_with(entity_id + 1, || None);
            v[entity_id] = Some(creator());
            self.components[T::INDEX] = Some(Box::new(Rc::new(RefCell::new(v))));
        }
    }

    pub fn get_container<T: 'static + Component>(&self) -> Option<ComponentContainerVec<T>> {
        let tid = T::INDEX;
        if let Some(x) = &self.components[tid] {
            if let Some(x) = x.as_any().downcast_ref::<ComponentContainerVec<T>>() {
                return Some(x.clone());
            }
        }
        None
    }

    pub fn get_component<T: 'static + Component + Clone>(&self, entity_id: EntityId) -> Option<T> {
        {
            let cache = self.entity_cache.borrow();
            if !cache.is_entity_alive(entity_id) {
                return None;
            }
        }

        if let Some(container) = self.get_container::<T>() {
            let container = container.deref().borrow();

            if let Some(component) = container.get(entity_id.0) {
                return component.clone();
            }
        }
        None
    }

    pub fn is_componet_exist<T: 'static + Component>(&self) -> bool {
        self.get_container::<T>().is_some()
    }

    pub fn visit_all<T: 'static + Component>(&self, f: impl Fn(&mut T)) {
        if let Some(cont_1) = self.get_container::<T>() {
            let mut c1 = cont_1.deref().borrow_mut();
            let iter = c1.iter_mut();
            let alive_check = {
                let cache = self.entity_cache.borrow();
                cache.get_alive_check()
            };

            iter.enumerate().for_each(|pair| {
                if alive_check.get(pair.0).unwrap() {
                    if let Some(a) = pair.1 {
                        f(a);
                    }
                }
            });
        }
    }

    pub fn visit_all2<A: 'static + Component, B: 'static + Component>(
        &self,
        f: impl Fn(&mut A, &mut B),
    ) {
        let cont_1 = self.get_container::<A>().unwrap();
        let cont_2 = self.get_container::<B>().unwrap();

        let mut c1 = cont_1.deref().borrow_mut();
        let mut c2 = cont_2.deref().borrow_mut();

        let len = std::cmp::min(c1.len(), c2.len());
        let alive_check = {
            let cache = self.entity_cache.borrow();
            cache.get_alive_check()
        };
        for i in 0..len {
            if !alive_check.get(i).unwrap() {
                continue;
            }
            if let (Some(a), Some(b)) = (c1.get_mut(i), c2.get_mut(i)) {
                if let (Some(a), Some(b)) = (a, b) {
                    f(a, b);
                }
            }
        }
    }

    pub fn visit_all3<A, B, C>(&self, f: impl Fn(&mut A, &mut B, &mut C))
    where
        A: 'static + Component,
        B: 'static + Component,
        C: 'static + Component,
    {
        let cont_1 = self.get_container::<A>().unwrap();
        let cont_2 = self.get_container::<B>().unwrap();
        let cont_3 = self.get_container::<C>().unwrap();

        let mut c1 = cont_1.deref().borrow_mut();
        let mut c2 = cont_2.deref().borrow_mut();
        let mut c3 = cont_3.deref().borrow_mut();

        let len = std::cmp::min(std::cmp::min(c1.len(), c2.len()), c3.len());
        let alive_check = {
            let cache = self.entity_cache.borrow();
            cache.get_alive_check()
        };
        for i in 0..len {
            if !alive_check.get(i).unwrap() {
                continue;
            }
            if let (Some(a), Some(b), Some(c)) = (c1.get_mut(i), c2.get_mut(i), c3.get_mut(i)) {
                if let (Some(a), Some(b), Some(c)) = (a, b, c) {
                    f(a, b, c);
                }
            }
        }
    }

    pub fn visit<T>(&self, entity_id: EntityId, f: impl FnOnce(&mut Option<T>))
    where
        T: 'static + Component,
    {
        {
            let cache = self.entity_cache.borrow();
            if !cache.is_entity_alive(entity_id) {
                return;
            }
        }

        let cont_1 = self.get_container::<T>().unwrap();
        let mut c1 = cont_1.deref().borrow_mut();
        if let Some(elem) = c1.get_mut(entity_id.0) {
            f(elem);
        }
    }

    pub fn visit2<A, B>(&self, entity: &Entity, f: impl Fn(&mut Option<A>, &mut Option<B>))
    where
        A: 'static + Component,
        B: 'static + Component,
    {
        {
            let cache = self.entity_cache.borrow();
            if !cache.is_entity_alive(entity.entity_id) {
                return;
            }
        }

        let cont_1 = self.get_container::<A>().unwrap();
        let cont_2 = self.get_container::<B>().unwrap();

        let mut c1 = cont_1.deref().borrow_mut();
        let mut c2 = cont_2.deref().borrow_mut();
        if let (Some(e1), Some(e2)) = (
            c1.get_mut(entity.entity_id.0),
            c2.get_mut(entity.entity_id.0),
        ) {
            f(e1, e2);
        }
    }

    pub fn process_self_events(&mut self) {

        let events = {
            let events = self.events.borrow();
            events.get_events::<EcsEvent>()
        };
        
        if let Some(events) = events {
            for ev in events {
                if let EcsEvent::EntityDestroyed(id) = ev {
                    let mut tmp = self.components.iter_mut();
                    while let Some(c) = tmp.next() {
                        if let Some(c) = c {
                            c.reset(id.0)
                        }
                    }
                }
            }
        }
    }
}