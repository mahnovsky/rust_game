use std::{collections::HashMap, fs};
use fxhash::FxHashMap;
use ecs::EntityWeak;
use glfw::{Key, Action};
use yaml_rust2::{Yaml, YamlLoader};
use std::str::FromStr;
use strum_macros::EnumString;

use crate::game::InputLayoutComponent;

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
        "LeftCtrl" => Some(Key::LeftControl),
        "RightCtrl" => Some(Key::RightControl),
    
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
    pub action: PlayerAction,
    config: PlayerConfig,
}

impl Player {
   pub fn new(entity: &EntityWeak, config: &str) -> Self {
        let config = PlayerConfig::new(config).unwrap();
        let input = config.get_input_component(entity.clone());
        if let Some(ent) = entity.upgrade() {
            ent.add_component(|| input);
        } 
        Self {
            entity: entity.clone(),
            action: PlayerAction::None,
            config,
        }
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
