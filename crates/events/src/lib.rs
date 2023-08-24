use std::default;
use std::env::Args;
use std::marker::PhantomData;
use std::ops::FnMut;
use std::{any::Any, any::TypeId, collections::HashMap};

#[derive(Clone)]
struct MyEvent {
    pub message: String,
    pub random_value: f32,
}

trait EventTrait<E>: FnMut(&E) {
    //fn get_type_id(&self) -> TypeId;

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

    fn add_receiver<E: Sized + 'static, Func: StoredFunc<E> + 'static>(
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

    fn push_event<E: Sized + 'static>(&mut self, event: &E) {
        let id = TypeId::of::<E>();

        if let Some(stor) = self.storages.get_mut(&id) {
            if let Some(stor) = stor.as_any_mut().downcast_mut::<Storage<E>>() {
                for receiver in stor.list.iter_mut() {
                    receiver.run(event);
                }
            }
        }
    }

    fn get_events<E: Clone + Sized + 'static>(&mut self, index: usize) -> Option<Vec<E>> {
        let id = TypeId::of::<E>();

        if let Some(stor) = self.storages.get_mut(&id) {
            if let Some(stor) = stor.as_any_mut().downcast_mut::<Storage<E>>() {
                return stor.list.get_mut(index)?.get_events();
            }
        }
        None
    }

    fn flush(&mut self) {
        for (_, stor) in &mut self.storages {
            stor.flush();
        }
    }
}

fn process_event(event: &MyEvent) {
    let MyEvent {
        message,
        random_value,
    } = event;
    println!("MyEvent on procced {message}, {random_value}");
}

struct World {
    my_event_handle: Option<usize>,
    mouse_event_handle: Option<usize>,
}

impl World {
    fn new() -> Self {
        Self {
            my_event_handle: None,
            mouse_event_handle: None,
        }
    }

    fn subscribe(&mut self, stor: &mut Events) {
        self.my_event_handle = stor.add_receiver(EventListener::<MyEvent>::new());
        self.mouse_event_handle = stor.add_receiver(EventListener::<MouseEvent>::new());
    }

    fn process_my_event(&mut self, stor: &mut Events) {
        if let Some(handle) = self.my_event_handle {
            let events = stor.get_events::<MyEvent>(handle).unwrap();

            for event in &events {
                let MyEvent {
                    message,
                    random_value,
                } = event;
                println!("Process events from MyEvent {message}, {random_value}");
            }
        }
    }

    fn process_mouse_event(&mut self, stor: &mut Events) {
        if let Some(handle) = self.mouse_event_handle {
            let events = stor.get_events::<MouseEvent>(handle).unwrap();

            for event in &events {
                let MouseEvent { x, y } = event;
                println!("Process events from MouseEvent {x}, {y}");
            }
        }
    }

    fn update(&mut self, stor: &mut Events) {
        self.process_my_event(stor);
        self.process_mouse_event(stor);
    }
}

struct EventListener<E> {
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

#[derive(Clone)]
struct MouseEvent {
    x: f32,
    y: f32,
}

fn process_mouse_event(me: &MouseEvent) {
    let MouseEvent { x, y } = me;
    println!("Mouse event processing {x}, {y}");
}

struct Game {
    world: World,
    events: Events,
}

impl Game {
    fn new() -> Self {
        let mut events = Events::new();
        let mut world = World::new();

        world.subscribe(&mut events);

        Self { world, events }
    }

    fn update(&mut self) {
        self.world.update(&mut self.events);

        self.events.flush();
    }

    fn push_event<E: Clone + Sized + 'static>(&mut self, event: &E) {
        self.events.push_event(event);
    }
}

fn main() {
    let mut game = Game::new();

    game.push_event(&MyEvent {
        message: "hello".to_owned(),
        random_value: 12_f32,
    });

    game.push_event(&MyEvent {
        message: "test".to_owned(),
        random_value: 32_f32,
    });

    game.push_event(&MouseEvent { x: 0.32, y: 12.22 });

    game.update();
}
