use ecs::EcsRc;

pub trait System {
    fn update(&mut self, world: &EcsRc, delta: f32);
}
