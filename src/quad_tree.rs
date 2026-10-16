use crate::bounds::Bounds;
use crate::collider2d::Collider2d;
use legion::*;
use std::collections::{self, VecDeque};
use std::ops::{Index, IndexMut};

const CHUNKS: usize = 4;
const MAX_DEPTH: u32 = 3;

const fn pow(base: i32, deg: i32) -> i32 {
    if deg <= 0 {
        return 1;
    }

    base * pow(base, deg - 1)
}

const fn get_count(level: i32) -> u8 {
    if level <= 0 {
        return 0;
    }
    let base = CHUNKS as i32;
    pow(base, level) as u8 + get_count(level - 1)
}
const MAX_NODE_COUNT_U8: u8 = get_count(MAX_DEPTH as i32);
const MAX_NODE_COUNT: usize = MAX_NODE_COUNT_U8 as usize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct CollisionObjectId(usize);

impl CollisionObjectId {
    pub const INVALID: Self = Self(usize::MAX);
    pub const WORLD_EDGE: Self = Self(usize::MAX - 1);
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Handle {
    index: u8,
}

impl Handle {
    const INVALID: Self = Self { index: u8::MAX };

    fn new(index: u8) -> Self {
        Self { index }
    }

    fn as_index(&self) -> usize {
        self.index as usize
    }
}

#[derive(Debug, Default)]
pub struct AreaNode {
    handle: Handle,
    parent: Handle,
    children: Option<[Handle; CHUNKS]>,
    bounds: Bounds,
    objects: Vec<CollisionObjectId>,
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

    fn insert_object(&mut self, object_id: CollisionObjectId) {
        //println!("Object {:?} enter area {:?}", object_id, self.handle.index);
        if !self.objects.contains(&object_id) {
            self.objects.push(object_id);
        }
    }

    fn remove_object(&mut self, object_id: CollisionObjectId) {
        //println!("Object {:?} leave area {:?}", object_id, self.handle.index);
        if let Some(index) = self.objects.iter().position(|x| *x == object_id) {
            _ = self.objects.swap_remove(index);
        }
    }
}

pub struct AreaMap<const N: usize>([AreaNode; N]);

impl<const N: usize> AreaMap<N> {
    pub fn get(&self, handle: Handle) -> Option<&AreaNode> {
        self.0.get(handle.as_index())
    }

    pub fn get_mut(&mut self, handle: Handle) -> Option<&mut AreaNode> {
        self.0.get_mut(handle.as_index())
    }
}

impl<const N: usize> Default for AreaMap<N> {
    fn default() -> Self {
        Self(std::array::from_fn(|_| Default::default()))
    }
}

impl<const N: usize> Index<Handle> for AreaMap<N> {
    type Output = AreaNode;

    fn index(&self, index: Handle) -> &Self::Output {
        &self.0[index.as_index()]
    }
}

impl<const N: usize> IndexMut<Handle> for AreaMap<N> {
    fn index_mut(&mut self, index: Handle) -> &mut Self::Output {
        &mut self.0[index.as_index()]
    }
}

#[derive(Debug, Clone)]
pub struct MoveResult {
    pub collisions: Vec<CollisionObjectId>,
    pub reached_world_edge: bool,
    pub bounds: Bounds,
}

pub struct QuadTree {
    objects: Vec<Option<Collider2d>>,
    nodes: AreaMap<MAX_NODE_COUNT>,
    tail: u8,
    bounds: Bounds,
    root: Handle,
}

#[allow(dead_code)]
impl QuadTree {
    pub fn new(bounds: Bounds) -> Self {
        Self {
            objects: Vec::new(),
            nodes: Default::default(),
            tail: 0,
            bounds,
            root: Default::default(),
        }
        .build_tree()
    }

    fn allocate(&mut self) -> Handle {
        let next_index = self.tail;
        self.tail += 1;
        Handle::new(next_index)
    }

    fn insert(&mut self, handle: Handle, node: AreaNode) {
        assert!(handle.index < self.tail);
        self.nodes[handle] = node;
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
            objects: Vec::new(),
        };
        self.insert(handle, node);

        handle
    }

    fn build_tree(mut self) -> Self {
        self.root = self.create_node(Handle::INVALID, 0, self.bounds);

        self
    }

    fn find_child_node_if(
        &self,
        handle: Handle,
        f: impl Fn(Option<&AreaNode>) -> bool,
    ) -> Option<Handle> {
        std::debug_assert!(handle.as_index() < MAX_NODE_COUNT);
        let node = self.nodes.get(handle)?;

        if let Some(children) = node.children {
            for child_handle in children {
                let child = self.nodes.get(child_handle);

                if f(child) {
                    return Some(child_handle);
                }
            }
        }

        None
    }

    fn get_place_node(&self, bounds: &Bounds) -> Option<Handle> {
        let mut next_handle = Some(self.root);
        let mut prev_handle = next_handle;
        while let Some(handle) = next_handle {
            let node = &self.nodes[handle];

            next_handle = node.find_last(self, bounds);
            if next_handle == prev_handle {
                break;
            }
            prev_handle = next_handle;
        }

        prev_handle
    }

    pub fn on_entity_removed(&mut self, id: CollisionObjectId, handle: Option<Handle>) {
        if let Some(handle) = handle {
            self.nodes[handle].remove_object(id);
        }
    }

    fn add_new_object(&mut self, collider: &mut Collider2d) -> CollisionObjectId {
        let id = if let Some(slot) = self
            .objects
            .iter_mut()
            .enumerate()
            .find(|(index, x)| x.is_none())
        {
            _ = slot.1.insert(collider.clone());

            CollisionObjectId(slot.0)
        } else {
            let id = CollisionObjectId(self.objects.len());
            self.objects.push(Some(collider.clone()));

            id
        };

        collider.set_tree_id(id);
        id
    }

    pub fn can_place(&self, world: &World, bounds: &Bounds) -> bool {
        let mut collisions = Vec::new();
        !self.find_collisions(&[], bounds, &mut collisions)
    }

    pub fn place(&mut self, collider: &mut Collider2d) -> bool {
        let bounds = collider.get_bounds();
        let mut collisions = Vec::new();
        self.find_collisions(&[], &bounds, &mut collisions);

        if let Some(handle) = self.get_place_node(&bounds) {
            let id = self.add_new_object(collider);

            self.nodes[handle].insert_object(id);
            collider.set_area_handle(Some(handle));

            return true;
        }

        false
    }

    fn find_collisions(
        &self,
        ignored: &[CollisionObjectId],
        bounds: &Bounds,
        collisions: &mut Vec<CollisionObjectId>,
    ) -> bool {
        let mut next_handle = Some(self.root);
        let mut check_nodes = VecDeque::new();
        let collisions_before = collisions.len();

        while let Some(handle) = next_handle {
            let node = &self.nodes[handle];

            for &id in &node.objects {
                if ignored.contains(&id) {
                    continue;
                }
                if let Some(Some(x)) = self.objects.get(id.0) {
                    if x.get_bounds().has_collision(bounds) {
                        collisions.push(id);
                    }
                }
            }

            if let Some(node_children) = node.children {
                for child_handle in node_children {
                    if self.nodes[child_handle].bounds.has_collision(bounds) {
                        check_nodes.push_back(child_handle);
                    }
                }
            }
            next_handle = check_nodes.pop_front();
        }
        collisions.len() > collisions_before
    }

    pub fn move_object(
        &mut self,
        entity: Entity,
        new_pos: glm::Vec2,
        collider: &mut Collider2d,
    ) -> MoveResult {
        let id = collider.get_tree_id();
        assert!(id != CollisionObjectId::INVALID);

        let mut bounds = collider.get_bounds();
        bounds.set_center_position(new_pos.x, new_pos.y);

        let mut collisions = Vec::new();
        let handle = self.get_place_node(&bounds);

        if let Some(handle) = handle {
            let has_collision = self.find_collisions(&[id], &bounds, &mut collisions);
            if has_collision {
                return MoveResult {
                    collisions,
                    reached_world_edge: false,
                    bounds,
                };
            }

            if let Some(old_handle) = collider.get_area_handle() {
                if old_handle != handle {
                    self.nodes[old_handle].remove_object(id);
                    self.nodes[handle].insert_object(id);
                }
            }

            collider.set_position(new_pos.x, new_pos.y);
            collider.set_area_handle(Some(handle));
            self.objects[id.0] = Some(collider.clone());

            return MoveResult {
                collisions,
                reached_world_edge: false,
                bounds,
            };
        }
        //collider.set_reached_border(true);

        MoveResult {
            collisions,
            reached_world_edge: true,
            bounds,
        }
    }

    /// Повна синхронізація дерева з ECS світом.
    /// Очищає об'єкти з усіх вузлів і будує індексацію на основі актуальних позицій.
    pub fn update(&mut self, world: &World) {
        // 1. Повністю очищаємо списки об'єктів у всіх вузлах дерева
        for i in 0..(self.tail as usize) {
            self.nodes.0[i].objects.clear();
        }
        self.objects.fill(None);
        // 2. Наново заповнюємо дерево та масив актуальними даними з ECS
        let mut query = <(Entity, &Collider2d)>::query();
        // Тут ми використовуємо небезпечне приведення або mut_cast, але оскільки це ECS update,
        // нам потрібно оновити внутрішні ID у самих компонентів, якщо вони нові, або зчитати старі.
        // Для спрощення логіки: вважаємо, що компоненти вже мають або отримують правильні ID.
        for (_ent, collider) in query.iter(world) {
            let id = collider.get_tree_id();
            if id == CollisionObjectId::INVALID {
                continue;
            }
            // Записуємо копію колайдера в глобальний масив дерева
            if id.0 >= self.objects.len() {
                self.objects.resize(id.0 + 1, None);
            }
            self.objects[id.0] = Some(collider.clone());
            // Кладемо об'єкт у правильний вузол дерева на основі його поточних bounds
            if let Some(handle) = self.get_place_node(&collider.get_bounds()) {
                self.nodes[handle].insert_object(id);
            }
        }
    }
}
