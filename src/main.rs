use device_query::{DeviceQuery, DeviceState, keymap::Keycode};
use dirs::config_dir;
use enigo::*;
use std::{fs, path::PathBuf, thread, time::{Duration, Instant}};
use serde::Deserialize;

// Configurate keycodes
const META_KEY: Keycode = Keycode::Meta;
const MOVE_LEFT_KEY: Keycode = Keycode::Left;
const MOVE_RIGHT_KEY: Keycode = Keycode::Right;
const MOVE_UP_KEY: Keycode = Keycode::Up;
const MOVE_DOWN_KEY: Keycode = Keycode::Down;
const MOUSE_LEFT_CLICK_KEY: Keycode = Keycode::RControl;
const MOUSE_RIGHT_CLICK_KEY: Keycode = Keycode::RShift;

#[derive(Deserialize, Default)]
struct Config {
    base_distance: Option<i32>,
    acceleration_factor: Option<f64>,
    max_distance: Option<i32>,
    sleep_duration: Option<u64>,
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
}

fn main() {
    let config = Config::load();
    let sleep_duration = Duration::from_millis(config.sleep_duration.unwrap_or(10));
    let device_state = DeviceState::new();
    let mut enigo = Enigo::new();
    let mut direction_state = DirectionState::new();
    let mut mouse_state = MouseState::new();


    loop {
        let keys = device_state.get_keys();
        let now = Instant::now();

        if keys.contains(&META_KEY) {
            handle_mouse_actions(&mut enigo, &keys, &mut mouse_state);
            let directions = detect_directions(&keys);
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

fn detect_directions(keys: &[Keycode]) -> Vec<Direction> {
    let mut directions = Vec::new();
    if keys.contains(&MOVE_LEFT_KEY) {
        directions.push(Direction::Left);
    }
    if keys.contains(&MOVE_RIGHT_KEY) {
        directions.push(Direction::Right);
    }
    if keys.contains(&MOVE_UP_KEY) {
        directions.push(Direction::Up);
    }
    if keys.contains(&MOVE_DOWN_KEY) {
        directions.push(Direction::Down);
    }
    directions
}

fn handle_mouse_actions(enigo: &mut Enigo, keys: &[Keycode], mouse_state: &mut MouseState) {
    // Update mouse button state based on current keys pressed
    mouse_state.update(enigo, keys);
}


// Adjust where `move_mouse` is called to pass `config` as well.
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

// Configuration for key mappings
   fn update(&mut self, enigo: &mut Enigo, keys: &[Keycode]) {

        let left = keys.contains(&MOUSE_LEFT_CLICK_KEY);
        let right = keys.contains(&MOUSE_RIGHT_CLICK_KEY);

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

#[derive(PartialEq, Clone, Copy)]
enum Direction {
    Left,
    Right,
    Up,
    Down,
}
