use std::ops::FnMut;
use std::{any::Any, any::TypeId, collections::HashMap};

#[derive(Clone)]
struct MyEvent {
    pub message: String,
    pub random_value: f32,
}

trait EventTrait<E>: FnMut(&E) {
    fn process(&mut self, event: &E);
}

trait BaseStorage {
    fn get_stored_type_id(&self) -> TypeId;

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn flush(&mut self);
}

trait StoredFunc<Event> {
    fn run(&mut self, input: &Event);

    fn get_events(&mut self) -> Option<Vec<Event>> {
        None
    }

    fn flush_events(&mut self) {}
}

impl<Event: 'static + Sized, Func: 'static + FnMut(&Event)> StoredFunc<Event> for Func {
    fn run(&mut self, input: &Event) {
        self(input);
    }
}

struct Storage<E> {
    pub list: Vec<Box<dyn StoredFunc<E>>>,
}

impl<E: Sized + 'static> Storage<E> {
    fn new() -> Self {
        Self { list: Vec::new() }
    }
    fn add(&mut self, item: Box<dyn StoredFunc<E>>) -> usize {
        let index = self.list.len();
        self.list.push(item);

        index
    }
}

impl<E: Sized + 'static> BaseStorage for Storage<E> {
    fn get_stored_type_id(&self) -> TypeId {
        TypeId::of::<E>()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn flush(&mut self) {
        for ev in &mut self.list {
            ev.flush_events();
        }
    }
}

struct Events {
    storages: HashMap<TypeId, Box<dyn BaseStorage>>,
}

impl Events {
    fn new() -> Self {
        Self {
            storages: HashMap::new(),
        }
    }

    pub fn add_receiver<E: Sized + 'static, Func: StoredFunc<E> + 'static>(
        &mut self,
        f: Func,
    ) -> Option<usize> {
        let id = TypeId::of::<E>();
        let index = self
            .storages
            .entry(id)
            .or_insert_with(|| Box::new(Storage::<E>::new()));

        let stor = index.as_any_mut();
        if let Some(stor) = stor.downcast_mut::<Storage<E>>() {
            return Some(stor.add(Box::new(f)));
        }

        None
    }

    pub fn push_event<E: Sized + 'static>(&mut self, event: &E) {
        let id = TypeId::of::<E>();

        if let Some(stor) = self.storages.get_mut(&id) {
            if let Some(stor) = stor.as_any_mut().downcast_mut::<Storage<E>>() {
                for receiver in stor.list.iter_mut() {
                    receiver.run(event);
                }
            }
        }
    }

    pub fn get_events<E: Clone + Sized + 'static>(&mut self, index: usize) -> Option<Vec<E>> {
        let id = TypeId::of::<E>();

        if let Some(stor) = self.storages.get_mut(&id) {
            if let Some(stor) = stor.as_any_mut().downcast_mut::<Storage<E>>() {
                return stor.list.get_mut(index)?.get_events();
            }
        }
        None
    }

    pub fn flush(&mut self) {
        for (_, stor) in &mut self.storages {
            stor.flush();
        }
    }
}

pub struct EventListener<E> {
    events: Vec<E>,
}

impl<E: 'static + Sized + Clone> EventListener<E> {
    fn new() -> Self {
        Self { events: Vec::new() }
    }
}

impl<E: 'static + Sized + Clone> StoredFunc<E> for EventListener<E> {
    fn run(&mut self, input: &E) {
        self.events.push(input.clone());
    }

    fn get_events(&mut self) -> Option<Vec<E>> {
        Some(self.events.clone())
    }

    fn flush_events(&mut self) {
        self.events.clear();
    }
}
