use crate::bounds::Bounds;
use crate::collider2d::Collider2d;
use ::ecs::*;
use std::cell::RefCell;
use std::collections::HashSet;
use std::ops::Deref;

const NODE_SIZE: f32 = 128.;

struct MapCell {
    index: usize,
    bounds: Option<Bounds>,
    objects: RefCell<HashSet<Entity>>,
}

impl MapCell {
    fn new() -> Self {
        Self {
            index: 0,
            bounds: None,
            objects: RefCell::new(HashSet::new()),
        }
    }

    fn init(&mut self, index: usize, bounds: Bounds) {
        self.index = index;
        self.bounds = Some(bounds);
    }
}

pub struct Map {
    size_x: u32,
    size_y: u32,
    cells: Box<[MapCell]>,
}

impl Map {
    pub fn new(width: u32, height: u32) -> Self {
        let (size_x, size_y, cells) = Self::create_cells(width, height);

        Self {
            size_x,
            size_y,
            cells: cells.into_boxed_slice(),
        }
    }

    fn create_cells(width: u32, height: u32) -> (u32, u32, Vec<MapCell>) {
        let size_x = width / NODE_SIZE as u32;
        let size_y = height / NODE_SIZE as u32;
        let capacity = (size_x * size_x) as usize;
        let mut cells = Vec::with_capacity(capacity);
        cells.resize_with(capacity, MapCell::new);

        for y in 0..size_y {
            for x in 0..size_x {
                let px = x as f32 * NODE_SIZE;
                let py = y as f32 * NODE_SIZE;
                let index = (x + y * size_x) as usize;
                cells[index].init(index, Bounds::new(px, py, NODE_SIZE, NODE_SIZE));
            }
        }

        (size_x, size_y, cells)
    }

    fn get_spawn_pos(&self) -> (f32, f32) {
        (0_f32, 0_f32)
    }

    pub fn try_place(&self, entity: Entity, bounds: Bounds) -> bool {
        let (x, y) = bounds.get_position();

        let (ix, iy) = (
            (x / NODE_SIZE).floor() as u32,
            (y / NODE_SIZE).floor() as u32,
        );

        if ix < self.size_x && iy < self.size_y {
            let check_cells = [(0, 0), (0, 1), (1, 1), (1, 0)];

            for (ox, oy) in check_cells.iter() {
                let cix = ix + ox;
                let ciy = iy + oy;
                if cix < self.size_x && ciy < self.size_y {
                    let index = (cix + ciy * self.size_x) as usize;
                    let checked_cell = &self.cells[index];
                    if let Some(cell_bounds) = &checked_cell.bounds {
                        if cell_bounds.has_collision(&bounds) {
                            let mut objects = checked_cell.objects.borrow_mut();

                            for ent in objects.iter() {
                                if let Some(collider) = ent.get_component_clone::<Collider2d>() {
                                    let b = collider.get_bounds();
                                    if bounds.has_collision(&b) {
                                        return false;
                                    }
                                }
                            }
                            objects.insert(entity.clone());
                        }
                    }
                }
            }

            return true;
        }

        false
    }

    pub fn move_object(&mut self, ecs: &EcsRc, entity_id: usize, new_pos: &glm::Vec2) -> bool {
        false
    }

    pub fn update(&mut self, world: &EcsRc) {
        let ecs = world.borrow();
        if let Some(colliders) = ecs.get_container::<Collider2d>() {
            let colliders = colliders.deref().borrow();
            for collider in colliders.iter().filter_map(|x| x.as_ref()) {
                //println!("place b {:?}", bounds);
                //self.try_place(collider.entity.clone(), collider.get_bounds());
            }
        }
        /*
        ecs.visit_all(|collider: &mut Option<Collider2d>| {
            if let Some(collider) = collider {
                let bounds = collider.get_bounds();
                //println!("place b {:?}", bounds);
                self.try_place(collider.entity, bounds);
            }
        });*/
    }
}
