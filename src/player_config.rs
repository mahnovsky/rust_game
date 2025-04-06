use std::cell::RefMut;
use std::{collections::HashMap, fs};
use fxhash::FxHashMap;
use ecs::EntityWeak;
use glfw::{Key, Action};
use yaml_rust2::{Yaml, YamlLoader};
use std::str::FromStr;
use strum_macros::EnumString;
use ::ecs::*;
use ecs_derive::component_impl;
use crate::bounds::Bounds;
use crate::collider2d::Collider2d;
use crate::quad_tree::QuadTree;
use crate::object_components::{Bullet, Damagable, Gun, Lifetime, Movable};
use crate::render::Render;
use crate::sprite::Sprite;
use crate::transform::Transform;
use crate::game::InputLayoutComponent;


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlayerState {
    Idle,
    Move
}

#[component_impl]
#[derive(Debug, Clone)]
pub struct PlayerController {
    pub direction: glm::Vec2,
    pub player_index: u32,
    pub state: PlayerState,
}

impl Listener<PlayerAction> for PlayerController {
    fn on_event(&mut self, player_input: PlayerAction) {
        self.direction = match player_input {
            PlayerAction::MoveLeft => glm::vec2(-1., 0.),
            PlayerAction::MoveRight => glm::vec2(1., 0.),
            PlayerAction::MoveTop => glm::vec2(0., 1.),
            PlayerAction::MoveDown => glm::vec2(0., -1.),
            PlayerAction::None | PlayerAction::Shoot => self.direction
        };

        self.state = match player_input {
            PlayerAction::MoveLeft |
            PlayerAction::MoveRight |
            PlayerAction::MoveTop |
            PlayerAction::MoveDown => PlayerState::Move,
            PlayerAction::Shoot => self.state,
            _ => PlayerState::Idle
        };

        //println!("Current state {:?}", self.state);
    }
}

impl PlayerController {
    fn new(entity: &EntityWeak, player_index: u32, dir: glm::Vec2) -> Self {
        Self {
            entity: entity.clone(),
            direction: glm::vec2(dir.x, dir.y),
            player_index,
            state: PlayerState::Idle,
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, EnumString)]
pub enum PlayerAction {
    MoveLeft,
    MoveRight,
    MoveTop,
    MoveDown,
    Shoot,
    None,
}

pub struct PlayerConfig {
    key_actions: FxHashMap<Key, PlayerAction>,
}

fn map_key(key: &Yaml) -> Option<Key> {
    let name = key.as_str()?;
    println!("{:?}", name);
    match name {
        "A" => Some(Key::A),
        "S" => Some(Key::S),
        "D" => Some(Key::D),
        "W" => Some(Key::W),
        "Left" => Some(Key::Left),
        "Right" => Some(Key::Right),
        "Up" => Some(Key::Up),
        "Down" => Some(Key::Down),
        "LeftControl" => Some(Key::LeftControl),
        "RightControl" => Some(Key::RightControl),
        "RightAlt" => Some(Key::RightAlt),
    
        _ => None,
    }
}

fn map_action(action: &Yaml) -> Option<PlayerAction> {
    let name = action.as_str()?;
    
    PlayerAction::from_str(name).ok()
}

fn parse_key_actions(doc: &Yaml) -> Option<FxHashMap<Key, PlayerAction>> {
    println!("Test ******* {}", doc["key_actions"].is_array());
    if let Some(keys) = doc["key_actions"].as_vec() {
        
        let mut res = FxHashMap::<Key, PlayerAction>::with_hasher(fxhash::FxBuildHasher::new());
        for item in keys.iter() {
            println!("{:?}", item);
            let key = map_key(&item["key"]);
            let action = map_action(&item["action"]);
            if let (Some(k), Some(a)) = (key, action) {
                println!("{:?} {:?}", k, a);
                res.insert(k, a);
            }
        }

        return res.into();
    }

    None
} 

impl PlayerConfig {
    fn new(player_name: &str) -> Option<Self> {
        let content = fs::read_to_string(std::path::Path::new(player_name));
        
        if let Ok(content) = content {
            println!("---------- {}", content.len());
            let docs = YamlLoader::load_from_str(&content).unwrap();

            let doc = &docs[0];

            let actions = parse_key_actions(doc)?;

            return Self{ key_actions: actions }.into();
        }
        None
    }

    fn get_input_component(&self, entity: EntityWeak) -> InputLayoutComponent {
        InputLayoutComponent::new(entity, self.key_actions.clone())
    }
}

pub struct Player {
    entity: EntityWeak,
    //pub action: PlayerAction,
    config: PlayerConfig,
}

impl Player {
   pub fn new(ecs: &EcsRc, index: u32, config: &str, quad_tree: RefMut<'_, QuadTree>, render: &Render) -> Option<Self> {
        let config = PlayerConfig::new(config).unwrap();
        let entity_weak = Entity::new(ecs);
        let entity = entity_weak.upgrade()?;
        let dir = glm::vec2(0_f32, 1_f32);
        let pos = glm::vec2(50. + (900 * index) as f32, 100.);
        let size = 50_f32;

        let bounds = Bounds::with_center_position(pos.x, pos.y, size, size);

        entity.add_component(|| Sprite::new(&entity_weak, size, size, "tank.png"));
        entity.add_component(|| Transform::with_direction(&entity_weak, pos, dir));
        entity.add_component(|| Collider2d::new(&entity_weak, bounds));
        entity.add_component(|| Movable::new(&entity_weak, 200.));
        entity.add_component(|| Damagable::new(&entity_weak, 10));
        entity.add_component(|| PlayerController::new(&entity_weak, index, dir));
        entity.add_component(|| Gun::new(&entity_weak, 2));

        entity.visit(|sprite: &mut Option<Sprite>| sprite.as_mut().unwrap().init(render));
        
        let input = config.get_input_component(entity_weak.clone());
        entity.add_component(|| input);
        quad_tree.place( ecs.borrow(), &entity);
        Self {
            entity: entity_weak.clone(),
            //action: PlayerAction::None,
            config,
        }.into()
    }

    fn get_player_entity(&self) -> EntityWeak {
        self.entity.clone()
    }

    // fn do_input(&mut self, event: &glfw::WindowEvent) {
    //     if let glfw::WindowEvent::Key(in_key, _, Action::Press, _) = event {
    //         if let Some(item) = self.input_layer.iter().find(|x| x.0 == *in_key) {
    //             self.action = item.1;
    //         }
    //     } else if let glfw::WindowEvent::Key(in_key, _, Action::Release, _) = event {
    //         let res = self.input_layer.iter().find(|x| x.0 == *in_key);

    //         if res.is_some() {
    //             self.action = PlayerAction::None;
    //         }
    //     }
    // }
}
