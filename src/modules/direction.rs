use serde::{Deserialize, Serialize};

// Define the Direction enum (you can customize this as needed)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum Direction {
    Up,
    Right,
    Down,
    Left,
    Idle,
}

impl Direction {
    pub fn from_str(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "front" => Direction::Up,
            "back" => Direction::Down,
            "left" => Direction::Left,
            "right" => Direction::Right,
            "idle" => Direction::Idle,
            _ => Direction::Idle, // Default to Idle for unknown strings
        }
    }
}
pub fn calculate_movement(direction: &Direction, x: &i16, y: &i16) -> (i16, i16) {
    match direction {
        Direction::Up => (*x, y + 1),                 // Moving up increases y
        Direction::Down => (*x, y.saturating_sub(1)), // Moving down decreases y
        Direction::Left => (x.saturating_sub(1), *y), // Moving left decreases x
        Direction::Right => (x + 1, *y),              // Moving right increases x
        Direction::Idle => (*x, *y),                  // Idle means no movement
    }
}

pub fn rotate_direction(dir: &Direction, angle: i16) -> Direction {
    match dir {
        direction => match angle {
            90 => match direction {
                Direction::Up => Direction::Right,
                Direction::Right => Direction::Down,
                Direction::Down => Direction::Left,
                Direction::Left => Direction::Up,
                Direction::Idle => Direction::Idle, // Idle stays Idle
            },
            180 => match direction {
                Direction::Up => Direction::Down,
                Direction::Right => Direction::Left,
                Direction::Down => Direction::Up,
                Direction::Left => Direction::Right,
                Direction::Idle => Direction::Idle, // Idle stays Idle
            },
            270 => match direction {
                Direction::Up => Direction::Left,
                Direction::Right => Direction::Up,
                Direction::Down => Direction::Right,
                Direction::Left => Direction::Down,
                Direction::Idle => Direction::Idle, // Idle stays Idle
            },
            0 => direction.clone(), // No rotation
            _ => {
                println!("Unsupported rotation angle: {}", angle);
                direction.clone() // Return the original direction in case of unsupported angles
            }
        },
    }
}
