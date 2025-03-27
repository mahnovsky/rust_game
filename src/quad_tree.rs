use crate::bounds::Bounds;
use crate::collider2d::Collider2d;
use ecs::*;
use std::cell::{Cell, Ref, RefCell};
use std::collections::{HashSet, VecDeque};

const CHUNKS: usize = 4;
const MAX_DEPTH: u32 = 3;

const fn pow(base: i32, deg: i32) -> i32 {
    if deg <= 0 {
        return 1;
    }

    base * pow(base, deg - 1)
}

const fn get_count(level: i32) -> usize {
    if level <= 0 {
        return 0;
    }
    let base = CHUNKS as i32;
    pow(base, level) as usize + get_count(level - 1)
}
const MAX_NODE_COUNT: usize = get_count(MAX_DEPTH as i32);

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Handle {
    index: usize,
}

impl Handle {
    const INVALID: Self = Self { index: usize::MAX };

    fn new(index: usize) -> Self {
        Self { index }
    }
}

struct AreaNode {
    handle: Handle,
    parent: Handle,
    bounds: Bounds,
    children: Option<[Handle; CHUNKS]>,
    objects: RefCell<HashSet<EntityId>>,
}

impl AreaNode {
    fn find_last(&self, area: &QuadTree, bounds: &Bounds) -> Option<Handle> {
        if bounds.is_inside_other(&self.bounds) {
            let child_handle = area.find_child_node_if(self.handle, |child| {
                if let Some(child) = child {
                    return bounds.is_inside_other(&child.bounds);
                }
                false
            });

            return child_handle.or(Some(self.handle));
        }
        None
    }

    fn insert_object(&self, object_id: EntityId) {
        let mut objects = self.objects.borrow_mut();

        println!("Object {:?} enter area {:?}", object_id, self.handle.index);
        objects.insert(object_id);
    }

    fn remove_object(&self, object_id: EntityId) {
        let mut objects = self.objects.borrow_mut();
        println!("Object {:?} leave area {:?}", object_id, self.handle.index);
        objects.remove(&object_id);
    }
}

pub struct QuadTree {
    nodes: [Option<AreaNode>; MAX_NODE_COUNT],
    tail: Cell<usize>,
    bounds: Bounds,
    root: Option<Handle>,
}

#[allow(dead_code)]
impl QuadTree {
    pub fn new(bounds: Bounds) -> Self {
        Self {
            nodes: std::array::from_fn(|_| None),
            tail: Cell::new(0),
            bounds,
            root: None,
        }
        .build_tree()
    }

    fn allocate(&self) -> Handle {
        Handle::new(self.tail.replace(self.tail.get() + 1))
    }

    fn insert(&mut self, handle: Handle, node: AreaNode) {
        assert!(handle.index < self.tail.get());
        self.nodes[handle.index] = Some(node);
    }

    fn create_node(&mut self, parent: Handle, depth: u32, bounds: Bounds) -> Handle {
        let handle = self.allocate();
        let next_depth = depth + 1;
        let (pos_x, pos_y) = bounds.get_position();
        let (size_x, size_y) = bounds.get_half_size();

        let children = if next_depth < MAX_DEPTH {
            Some([
                self.create_node(
                    handle,
                    next_depth,
                    Bounds::new(pos_x, pos_y, size_x, size_y),
                ),
                self.create_node(
                    handle,
                    next_depth,
                    Bounds::new(pos_x, pos_y + size_y, size_x, size_y),
                ),
                self.create_node(
                    handle,
                    next_depth,
                    Bounds::new(pos_x + size_x, pos_y + size_y, size_x, size_y),
                ),
                self.create_node(
                    handle,
                    next_depth,
                    Bounds::new(pos_x + size_x, pos_y, size_x, size_y),
                ),
            ])
        } else {
            None
        };

        let node = AreaNode {
            handle,
            parent,
            bounds,
            children,
            objects: RefCell::new(HashSet::new()),
        };
        self.insert(handle, node);

        handle
    }

    fn build_tree(mut self) -> Self {
        self.root = Some(self.create_node(Handle::INVALID, 0, self.bounds.clone()));

        self
    }

    fn find_child_node_if(
        &self,
        handle: Handle,
        f: impl Fn(Option<&AreaNode>) -> bool,
    ) -> Option<Handle> {
        if let Some(Some(node)) = self.nodes.get(handle.index) {
            if let Some(children) = node.children.as_ref() {
                for child_handle in children {
                    if child_handle.index < self.nodes.len() {
                        let child = self.nodes.get(child_handle.index).unwrap();

                        if f(child.as_ref()) {
                            return Some(*child_handle);
                        }
                    }
                }
            }
        }
        None
    }

    fn get_place_node(&self, bounds: &Bounds) -> Option<Handle> {
        let nodes_count = self.nodes.len();
        let mut next_handle = self.root;
        let mut prev_handle = next_handle;
        while let Some(handle) = next_handle {
            if handle.index < nodes_count {
                let node = self.nodes[handle.index].as_ref().unwrap();

                next_handle = node.find_last(self, bounds);
                if next_handle == prev_handle {
                    break;
                }
                prev_handle = next_handle;
            } else {
                break;
            }
        }

        prev_handle
    }

    pub fn on_entity_removed(&self, id: EntityId) {
        let handle = self.find_child_node_if(self.root.unwrap(), |node| {
            if let Some(node) = node {
                let objects = node.objects.borrow();

                return objects.contains(&id);
            }
            false
        });
        if let Some(handle) = handle {
            if let Some(node) = &self.nodes[handle.index] {
                node.remove_object(id);
            }
        }
    }

    pub fn place(&self, entity: &Entity) {
        let collider = entity.get_component_clone::<Collider2d>();
        if let Some(collider) = collider {
            let bounds = collider.get_bounds();
            let handle = self.get_place_node(&bounds);

            if let Some(handle) = handle {
                let node = self.nodes[handle.index].as_ref().unwrap();

                node.insert_object(entity.get_id());
                entity.visit::<Collider2d>(|collider| {
                    if let Some(collider) = collider {
                        collider.set_area_handle(Some(handle));
                    }
                });
            }
        }
    }

    fn is_collisions_exist(
        &self,
        ecs: &Ref<'_, Ecs>,
        entity: &Entity,
        collider: &Collider2d,
        bounds: &Bounds,
    ) -> bool {
        let nodes_count = self.nodes.len();
        let mut next_handle = self.root;
        let mut check_nodes = VecDeque::new();
        while let Some(handle) = next_handle {
            if handle.index < nodes_count {
                let node = self.nodes[handle.index].as_ref().unwrap();
                for c in node.objects.borrow().iter().filter_map(|e| {
                    if *e != entity.get_id() && !collider.is_ignored_entity(*e) {
                        ecs.get_component::<Collider2d>(*e)
                    } else {
                        None
                    }
                }) {
                    if c.get_bounds().has_collision(bounds) {
                        return true;
                    }
                }
                if let Some(node_children) = node.children {
                    for child_handle in node_children {
                        let node = self.nodes[child_handle.index].as_ref().unwrap();
                        if node.bounds.has_collision(bounds) {
                            check_nodes.push_back(child_handle);
                        }
                    }
                }

                next_handle = check_nodes.pop_front();
            } else {
                break;
            }
        }

        false
    }

    pub fn move_object(&self, ecs: &Ref<'_, Ecs>, entity: &Entity, new_pos: glm::Vec2) -> bool {
        if let Some(collider) = ecs.get_component::<Collider2d>(entity.get_id()) {
            let mut bounds = collider.get_bounds();
            bounds.set_center_position(new_pos.x, new_pos.y);
            let handle = self.get_place_node(&bounds);

            if let Some(handle) = handle {
                if self.is_collisions_exist(ecs, entity, &collider, &bounds) {

                    return false;
                }
                if let Some(old_handle) = collider.get_area_handle() {
                    if old_handle != handle {
                        let node = self.nodes[handle.index].as_ref().unwrap();
                        node.insert_object(entity.get_id());
                        if let Some(node) = &self.nodes[old_handle.index] {
                            node.remove_object(entity.get_id());
                        }
                    }
                }

                ecs.visit::<Collider2d>(entity, |collider| {
                    if let Some(collider) = collider.as_mut() {
                        collider.set_position(new_pos.x, new_pos.y);
                        collider.set_reached_border(false);
                        collider.set_area_handle(Some(handle));
                    }
                });

                return true;
            }
        }
        ecs.visit::<Collider2d>(entity, |collider| {
            if let Some(collider) = collider.as_mut() {
                collider.set_reached_border(true);
            }
        });

        false
    }
}
