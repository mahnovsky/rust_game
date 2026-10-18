use crate::{
    bounds::Bounds,
    collider2d::{Collider2d, ColliderRegisterComponent},
    game::InputLayoutComponent,
    object_components::{
        Damagable, Direction, Gun, Movable,
        MoveSwitch::{self, Stay},
    },
    player_config::{PlayerAction::Shoot, PlayerState::Idle},
    quad_tree::QuadTree,
    sprite::Sprite,
    transform::Transform,
};
use fxhash::FxHashMap;
use glfw::{
    Key::{self},
    Window,
};
use legion::*;
use std::fs;
use std::str::FromStr;
use strum_macros::EnumString;
use yaml_rust2::{Yaml, YamlLoader};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlayerState {
    Idle,
    Move,
}

#[derive(Debug, Clone)]
pub struct PlayerController {
    pub player_index: u32,
    pub state: PlayerState,
    direction: Direction,
    input_layout: InputLayoutComponent,
}

impl PlayerController {
    fn new(player_index: u32, direction: Direction, input_layout: InputLayoutComponent) -> Self {
        Self {
            player_index,
            state: PlayerState::Idle,
            direction,
            input_layout,
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
    Idle,
}

pub struct PlayerConfig {
    key_actions: FxHashMap<Key, PlayerAction>,
}

fn map_key(key: &Yaml) -> Option<Key> {
    let name = key.as_str()?;
    println!("{:?}", name);
    let k = match name {
        "A" => Key::A,
        "S" => Key::S,
        "D" => Key::D,
        "W" => Key::W,
        "Left" => Key::Left,
        "Right" => Key::Right,
        "Up" => Key::Up,
        "Down" => Key::Down,
        "LeftControl" => Key::LeftControl,
        "RightControl" => Key::RightControl,
        "RightAlt" => Key::RightAlt,
        _ => return None,
    };

    Some(k)
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

            let Some(key) = map_key(&item["key"]) else {
                continue;
            };

            let Some(action) = map_action(&item["action"]) else {
                continue;
            };

            println!("{:?} {:?}", key, action);
            res.insert(key, action);
        }

        return res.into();
    }

    None
}

impl PlayerConfig {
    fn from_file(player_name: &str) -> Option<Self> {
        let content = fs::read_to_string(std::path::Path::new(player_name));

        if let Ok(content) = content {
            println!("---------- {}", content.len());
            let docs = YamlLoader::load_from_str(&content).unwrap();

            let doc = &docs[0];

            let actions = parse_key_actions(doc)?;

            return Self {
                key_actions: actions,
            }
            .into();
        }
        None
    }

    fn get_input_component(&self) -> InputLayoutComponent {
        InputLayoutComponent::new(self.key_actions.clone())
    }
}

pub fn spawn_player(
    world: &mut World,
    quad_tree: &mut QuadTree,
    index: u32,
    config_file: &str,
) -> Option<Entity> {
    let config = PlayerConfig::from_file(config_file)?;

    let dir = Direction::Up;
    let pos = glm::vec2(50. + (900 * index) as f32, 100.);
    let size = 50_f32;
    let bsize = size * 0.9;
    let bounds = Bounds::with_center_position(pos.x, pos.y, bsize, bsize);

    let entity = world.push((
        Sprite::new(size, size, "tank.png"),
        Transform::with_direction(pos, dir.to_vec()),
        ColliderRegisterComponent,
        Collider2d::new(bounds),
        Movable::new(200, dir, Stay),
        Damagable::new(10),
        PlayerController::new(index, dir, config.get_input_component()),
        Gun::new(2),
    ));

    Some(entity)
}
/*
pub struct Player {
    //pub action: PlayerAction,
    config: PlayerConfig,
}*/

impl PlayerController {
    /*pub fn new(
        world: &World,
        index: u32,
        config: &str,
        quad_tree: &mut QuadTree,
        render: &Render,
    ) -> Self {
        let config = PlayerConfig::new(config).unwrap();
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

        let input = config.get_input_component();

        entity.add_component(|| input);
        quad_tree.place();
        Self {
            //action: PlayerAction::None,
            config,
        }
    }*/

    fn update_state(&mut self, player_input: Option<PlayerAction>) {
        if let Some(player_input) = player_input {
            self.direction = match player_input {
                PlayerAction::MoveLeft => Direction::Left,
                PlayerAction::MoveRight => Direction::Right,
                PlayerAction::MoveTop => Direction::Up,
                PlayerAction::MoveDown => Direction::Down,
                PlayerAction::Idle | PlayerAction::Shoot => self.direction,
            };

            self.state = match player_input {
                PlayerAction::MoveLeft
                | PlayerAction::MoveRight
                | PlayerAction::MoveTop
                | PlayerAction::MoveDown => PlayerState::Move,
                PlayerAction::Shoot => self.state,
                _ => PlayerState::Idle,
            };
        } else {
            self.state = Idle;
        }
    }

    pub fn do_input(&mut self, event: &glfw::WindowEvent) {
        //if let Some(p) = self.input_layout.do_input(event) {
        //    println!("do_input action {:?}", p);
        //    self.update_state(p);
        //} else {
        //    self.update_state(PlayerAction::Idle);
        //}

        //if let glfw::WindowEvent::Key(in_key, _, Action::Press, _) = event {
        //    if let Some(item) = self.input_layout.iter().find(|x| x.0 == *in_key) {
        //        self.action = item.1;
        //    }
        //} else if let glfw::WindowEvent::Key(in_key, _, Action::Release, _) = event {
        //    let res = self.input_layer.iter().find(|x| x.0 == *in_key);

        //    if res.is_some() {
        //        self.action = PlayerAction::None;
        //    }
        //}
    }

    pub fn update(
        &mut self,
        window: &glfw::Window,
        ent: Entity,
        movable: &mut Movable,
        gun: &mut Gun,
        transform: &Transform,
    ) {
        let action = self.input_layout.scan_input(window);
        self.update_state(action);

        if let Some(action) = action {
            if action == Shoot && gun.can_spawn_bullet() {
                let pos = transform.get_position();
                let dir = movable.get_direction();
                gun.setup_shoot(ent, pos, dir);
            }
        }

        movable.set_direction(self.direction);
        let switch = match self.state {
            PlayerState::Move => MoveSwitch::Forward,
            PlayerState::Idle => MoveSwitch::Stay,
        };
        movable.set_move_switch(switch);
    }
}

pub fn update_players(world: &mut World, window: &Window) {
    let mut query = <(
        Entity,
        &mut PlayerController,
        &mut Movable,
        &mut Gun,
        &Transform,
    )>::query();

    for (ent, pc, movable, gun, tr) in query.iter_mut(world) {
        pc.update(window, *ent, movable, gun, tr);
    }
}
