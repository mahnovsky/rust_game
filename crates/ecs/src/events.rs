use std::rc::{Rc, Weak};
use std::{any::Any, any::TypeId, collections::HashMap};

pub trait EventListener {
    fn on_event(&mut self);
}

trait AsAny {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn clear(&mut self);
}

struct EventStorage<E> {
    events: Vec<E>,
}

impl<E: Sized + 'static + Clone> EventStorage<E> {
    fn new() -> Self {
        Self { events: Vec::new() }
    }

    fn push(&mut self, ev: E) {
        self.events.push(ev);
    }
}

impl<E: Sized + 'static + Clone> AsAny for EventStorage<E> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clear(&mut self) {
        self.events.clear();
    }
}

pub trait Listener<E> {
    fn on_event(&mut self, event: E);
}

pub struct EventSystem {
    storages: HashMap<TypeId, Box<dyn AsAny>>,
}

impl EventSystem {
    pub fn new() -> Self {
        Self {
            storages: HashMap::new(),
        }
    }

    fn get_storage<E: 'static + Sized + Clone>(&self) -> Option<&EventStorage<E>> {
        let id = TypeId::of::<E>();

        if let Some(storage) = self.storages.get(&id) {
            let any_stor = storage.as_any();
            return any_stor.downcast_ref::<EventStorage<E>>();
        }
        None
    }

    fn get_storage_mut<E: 'static + Sized + Clone>(&mut self) -> Option<&mut EventStorage<E>> {
        let id = TypeId::of::<E>();

        let storage = self
            .storages
            .entry(id)
            .or_insert_with(|| Box::new(EventStorage::<E>::new()));

        let any_stor = storage.as_any_mut();
        any_stor.downcast_mut::<EventStorage<E>>()
    }

    pub fn push_event<E: 'static + Sized + Clone>(&mut self, ev: E) {
        if let Some(storage) = self.get_storage_mut::<E>() {
            storage.push(ev);
        }
    }

    pub fn process_event<E: 'static + Sized + Clone>(&self, listener: &mut dyn Listener<E>) {
        if let Some(storage) = self.get_storage::<E>() {
            for ev in storage.events.iter() {
                listener.on_event(ev.clone());
            }
        }
    }

    pub fn clear_all(&mut self) {
        for (_, v) in &mut self.storages {
            v.clear();
        }
    }
}
