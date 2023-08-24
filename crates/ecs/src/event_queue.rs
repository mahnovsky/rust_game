use std::any::Any;

pub trait BaseListenerTrait {
    fn on_event(&self, e: &dyn EventTrait);
}

pub trait ListenerTrait<T: 'static + EventTrait>
where
    Self: BaseListenerTrait,
{
    fn on_event_t(&self, e: T);
}

pub trait EventTrait {
    fn get_event_id(&self) -> usize;
    fn as_any(&self) -> &dyn Any;
}

pub trait EventIndex {
    const INDEX: usize;
}

trait EventContainer {
    fn dispatch(&self, listeners: &[&dyn BaseListenerTrait]);

    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: EventTrait + 'static> EventContainer for Vec<T> {
    fn dispatch(&self, listeners: &[&dyn BaseListenerTrait]) {
        for e in self.iter() {
            listeners.iter().for_each(|s| s.on_event(e));
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Default)]
pub struct EventQueue {
    events: Vec<Box<dyn EventContainer>>,
}

impl EventQueue {
    pub fn push<T: EventTrait + EventIndex + 'static>(&mut self, event: T) {
        if self.events.len() <= T::INDEX {
            self.events
                .resize_with(T::INDEX + 1, || Box::<Vec<T>>::default());
        }

        let mut events = self.events.get_mut(T::INDEX);
        if let Some(events) = events.as_mut() {
            if let Some(events) = events.as_any_mut().downcast_mut::<Vec<T>>() {
                events.push(event);
            }
        }
    }

    pub fn dispatch(&mut self, listeners: &[&dyn BaseListenerTrait]) {
        self.events.iter().for_each(|e| e.dispatch(listeners));
    }

    pub fn flush<T: EventTrait + EventIndex + Clone + 'static>(&mut self) -> Option<Vec<T>> {
        let mut events = self.events.get_mut(T::INDEX);
        if let Some(events) = events.as_mut() {
            if let Some(events) = events.as_any_mut().downcast_mut::<Vec<T>>() {
                return Some(events.to_vec());
            }
        }
        None
    }
}
