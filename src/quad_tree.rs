use crate::bounds::Bounds;
use crate::collider2d::Collider2d;
use fxhash::{FxBuildHasher, FxHashMap, FxHashSet, FxHasher};
use legion::*;
use std::cell::{Cell, Ref, RefCell};
use std::collections::{HashSet, VecDeque};
use std::hash::Hash;

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
    objects: FxHashSet<Entity>,
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

    fn insert_object(&mut self, object_id: Entity) {
        println!("Object {:?} enter area {:?}", object_id, self.handle.index);
        self.objects.insert(object_id);
    }

    fn remove_object(&mut self, object_id: Entity) {
        println!("Object {:?} leave area {:?}", object_id, self.handle.index);
        self.objects.remove(&object_id);
    }
}

pub struct CollisionSummary {
    pub collide_ent: Option<Entity>,
    pub can_move: bool,
}

#[derive(Debug, Clone, Copy)]
struct Object {
    ent: Entity,
    bounds: Bounds,
}

impl Hash for Object {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.ent.hash(state);
    }
}

pub struct QuadTree {
    objects: FxHashMap<Entity, Collider2d>,
    nodes: [Option<AreaNode>; MAX_NODE_COUNT],
    tail: Cell<usize>,
    bounds: Bounds,
    root: Option<Handle>,
}

#[allow(dead_code)]
impl QuadTree {
    pub fn new(bounds: Bounds) -> Self {
        Self {
            objects: FxHashMap::with_hasher(FxBuildHasher::new()),
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
            objects: FxHashSet::with_hasher(FxBuildHasher::new()),
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

    pub fn on_entity_removed(&mut self, id: Entity) {
        let handle = self.find_child_node_if(self.root.unwrap(), |node| {
            if let Some(node) = node {
                return node.objects.contains(&id);
            }
            false
        });
        if let Some(handle) = handle {
            if let Some(node) = &mut self.nodes[handle.index] {
                node.remove_object(id);
            }
        }
    }

    pub fn can_place(&self, world: &World, bounds: &Bounds) -> bool {
        if let Some(id) = self.get_collision(None, None, &bounds) {
            return false;
        }
        true
    }

    pub fn place(&self, world: &mut World, entity: Entity) -> bool {
        let Some(entry) = world.entry(entity) else {
            return false;
        };

        let Ok(collider) = entry.get_component::<Collider2d>() else {
            return false;
        };

        let bounds = collider.get_bounds();
        if let Some(id) = self.get_collision(Some(entity), None, &bounds) {
            return false;
        }

        let handle = self.get_place_node(&bounds);

        if let Some(handle) = handle {
            let node = self.nodes[handle.index].as_ref().unwrap();

            /*node.insert_object(entity);
            entity.visit::<Collider2d>(|collider| {
                if let Some(collider) = collider {
                    collider.set_area_handle(Some(handle));
                }
            });*/

            return true;
        }

        return false;
    }

    fn get_collision(
        &self,
        entity_id: Option<Entity>,
        ignored: Option<&[Entity]>,
        bounds: &Bounds,
    ) -> Option<Entity> {
        let nodes_count = self.nodes.len();
        let mut next_handle = self.root;
        let mut check_nodes = VecDeque::new();

        while let Some(handle) = next_handle {
            if handle.index < nodes_count {
                let node = self.nodes[handle.index].as_ref().unwrap();
                let iter = node.objects.iter().filter(|e| {
                    let s = entity_id.iter().any(|ent| *ent == **e);
                    if let Some(ignore) = ignored.as_ref().map(|x| x.contains(e)) {
                        return !ignore && !s;
                    };

                    !s
                });

                for ent in iter {
                    let Some(collider) = self.objects.get(ent) else {
                        continue;
                    };

                    if collider.get_bounds().has_collision(bounds) {
                        return Some(*ent);
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

        None
    }

    pub fn move_object(&mut self, entity: Entity, new_pos: glm::Vec2) -> Option<CollisionSummary> {
        let mut collider = self.objects.get(&entity)?.clone();
        let mut bounds = collider.get_bounds();
        bounds.set_center_position(new_pos.x, new_pos.y);

        let handle = self.get_place_node(&bounds);

        if let Some(handle) = handle {
            let collide_ent = self.get_collision(Some(entity), None, &bounds);

            if collide_ent.is_some() {
                return Some(CollisionSummary {
                    collide_ent,
                    can_move: false,
                });
            }
            if let Some(old_handle) = collider.get_area_handle() {
                if old_handle != handle {
                    let node = self.nodes[handle.index].as_mut().unwrap();
                    node.insert_object(entity);
                    if let Some(node) = &mut self.nodes[old_handle.index] {
                        node.remove_object(entity);
                    }
                }
            }

            collider.set_position(new_pos.x, new_pos.y);
            collider.set_reached_border(false);
            collider.set_area_handle(Some(handle));

            return Some(CollisionSummary {
                collide_ent: None,
                can_move: true,
            });
        }

        collider.set_reached_border(true);

        Some(CollisionSummary {
            collide_ent: None,
            can_move: false,
        })
    }
}
