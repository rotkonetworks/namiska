
use device_query::{DeviceQuery, DeviceState, keymap::Keycode};
use dirs::config_dir;
use enigo::*;
use std::{fs, path::PathBuf, thread, time::{Duration, Instant}};
use serde::Deserialize;

#[derive(PartialEq, Clone, Copy)]
enum Direction {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Deserialize, Default)]
struct KeyConfig {
    meta: Option<String>,
    left: Option<String>,
    right: Option<String>,
    up: Option<String>,
    down: Option<String>,
    mouse_left: Option<String>,
    mouse_right: Option<String>,
}

#[derive(Deserialize, Default)]
struct Config {
    base_distance: Option<i32>,
    acceleration_factor: Option<f64>,
    max_distance: Option<i32>,
    sleep_duration: Option<u64>,
    keys: Option<KeyConfig>,
}

impl Config {
    fn load() -> Self {
        let path = config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("namiska/config.toml");

        fs::read_to_string(&path)
            .map(|contents| toml::from_str(&contents).unwrap_or_default())
            .unwrap_or_default()
    }

    fn get_key(&self, key_str: Option<&String>, default: Keycode) -> Keycode {
        key_str
            .and_then(|s| match s.to_uppercase().as_str() {
                "META" | "SUPER" | "WIN" => Some(Keycode::Meta),
                "CTRL" | "CONTROL" | "LCTRL" => Some(Keycode::LControl),
                "RCTRL" | "RCONTROL" => Some(Keycode::RControl),
                "ALT" | "LALT" => Some(Keycode::LAlt),
                "RALT" => Some(Keycode::RAlt),
                "SHIFT" | "LSHIFT" => Some(Keycode::LShift),
                "RSHIFT" => Some(Keycode::RShift),
                "UP" => Some(Keycode::Up),
                "DOWN" => Some(Keycode::Down),
                "LEFT" => Some(Keycode::Left),
                "RIGHT" => Some(Keycode::Right),
                _ => None
            })
        .unwrap_or(default)
    }

    fn meta_key(&self) -> Keycode {
        self.keys.as_ref()
            .and_then(|k| k.meta.as_ref())
            .map(|s| self.get_key(Some(s), Keycode::Meta))
            .unwrap_or(Keycode::Meta)
    }

    fn move_keys(&self) -> (Keycode, Keycode, Keycode, Keycode) {
        let default_config = KeyConfig::default();
        let keys = self.keys.as_ref().unwrap_or(&default_config);
        (
            self.get_key(keys.left.as_ref(), Keycode::Left),
            self.get_key(keys.right.as_ref(), Keycode::Right),
            self.get_key(keys.up.as_ref(), Keycode::Up),
            self.get_key(keys.down.as_ref(), Keycode::Down),
        )
    }

    fn mouse_keys(&self) -> (Keycode, Keycode) {
        let default_config = KeyConfig::default();
        let keys = self.keys.as_ref().unwrap_or(&default_config);
        (
            self.get_key(keys.mouse_left.as_ref(), Keycode::RControl),
            self.get_key(keys.mouse_right.as_ref(), Keycode::RShift),
        )
    }
}

struct KeyState {
    meta_key: Keycode,
    left_key: Keycode,
    right_key: Keycode,
    up_key: Keycode,
    down_key: Keycode,
    mouse_left_key: Keycode,
    mouse_right_key: Keycode,
}

impl KeyState {
    fn from_config(config: &Config) -> Self {
        let (left, right, up, down) = config.move_keys();
        let (mouse_left, mouse_right) = config.mouse_keys();
        Self {
            meta_key: config.meta_key(),
            left_key: left,
            right_key: right,
            up_key: up,
            down_key: down,
            mouse_left_key: mouse_left,
            mouse_right_key: mouse_right,
        }
    }
}

fn detect_directions(keys: &[Keycode], key_state: &KeyState) -> Vec<Direction> {
    let mut directions = Vec::new();
    if keys.contains(&key_state.left_key) {
        directions.push(Direction::Left);
    }
    if keys.contains(&key_state.right_key) {
        directions.push(Direction::Right);
    }
    if keys.contains(&key_state.up_key) {
        directions.push(Direction::Up);
    }
    if keys.contains(&key_state.down_key) {
        directions.push(Direction::Down);
    }
    directions
}

fn handle_mouse_actions(enigo: &mut Enigo, keys: &[Keycode], mouse_state: &mut MouseState, key_state: &KeyState) {
    mouse_state.update(enigo, keys, key_state);
}

fn move_mouse(enigo: &mut Enigo, direction: Direction, elapsed: Duration, config: &Config) {
    let distance = calculate_distance(config, elapsed.as_millis());
    match direction {
        Direction::Left => enigo.mouse_move_relative(-distance, 0),
        Direction::Right => enigo.mouse_move_relative(distance, 0),
        Direction::Up => enigo.mouse_move_relative(0, -distance),
        Direction::Down => enigo.mouse_move_relative(0, distance),
    }
}

fn calculate_distance(config: &Config, elapsed: u128) -> i32 {
    let base_distance = config.base_distance.unwrap_or(5);
    let acceleration_factor = config.acceleration_factor.unwrap_or(0.05);
    let max_distance = config.max_distance.unwrap_or(150);

    std::cmp::min(base_distance + (elapsed as f64 * acceleration_factor) as i32, max_distance)
}

struct DirectionState {
    last_press_time: Instant,
    current_directions: Vec<Direction>,
}

impl DirectionState {
    fn new() -> Self {
        Self {
            last_press_time: Instant::now(),
            current_directions: Vec::new(),
        }
    }

    fn update(&mut self, directions: &[Direction], now: Instant) {
        if &self.current_directions != directions {
            self.last_press_time = now;
            self.current_directions = directions.to_vec();
        }
    }

    fn reset(&mut self) {
        self.current_directions.clear();
    }

    fn calculate_elapsed(&self, now: Instant) -> Option<(Vec<Direction>, Duration)> {
        if !self.current_directions.is_empty() {
            Some((self.current_directions.clone(), now.duration_since(self.last_press_time)))
        } else {
            None
        }
    }
}

struct MouseState {
    left_pressed: bool,
    right_pressed: bool,
}

impl MouseState {
    fn new() -> Self {
        Self {
            left_pressed: false,
            right_pressed: false,
        }
    }

    fn update(&mut self, enigo: &mut Enigo, keys: &[Keycode], key_state: &KeyState) {
        let left = keys.contains(&key_state.mouse_left_key);
        let right = keys.contains(&key_state.mouse_right_key);

        if left && !self.left_pressed {
            enigo.mouse_down(MouseButton::Left);
            self.left_pressed = true;
        } else if !left && self.left_pressed {
            enigo.mouse_up(MouseButton::Left);
            self.left_pressed = false;
        }

        if right && !self.right_pressed {
            enigo.mouse_down(MouseButton::Right);
            self.right_pressed = true;
        } else if !right && self.right_pressed {
            enigo.mouse_up(MouseButton::Right);
            self.right_pressed = false;
        }
    }

    fn reset(&mut self, enigo: &mut Enigo) {
        if self.left_pressed {
            enigo.mouse_up(MouseButton::Left);
            self.left_pressed = false;
        }
        if self.right_pressed {
            enigo.mouse_up(MouseButton::Right);
            self.right_pressed = false;
        }
    }
}

fn main() {
    let config = Config::load();
    let key_state = KeyState::from_config(&config);
    let sleep_duration = Duration::from_millis(config.sleep_duration.unwrap_or(10));
    let device_state = DeviceState::new();
    let mut enigo = Enigo::new();
    let mut direction_state = DirectionState::new();
    let mut mouse_state = MouseState::new();

    loop {
        let keys = device_state.get_keys();
        let now = Instant::now();

        if keys.contains(&key_state.meta_key) {
            handle_mouse_actions(&mut enigo, &keys, &mut mouse_state, &key_state);
            let directions = detect_directions(&keys, &key_state);
            if !directions.is_empty() {
                direction_state.update(&directions, now);
            } else {
                direction_state.reset();
            }
        } else {
            direction_state.reset();
            mouse_state.reset(&mut enigo);
        }

        if let Some((directions, elapsed)) = direction_state.calculate_elapsed(now) {
            for direction in directions {
                move_mouse(&mut enigo, direction, elapsed, &config);
            }
        }

        thread::sleep(sleep_duration);
    }
}
