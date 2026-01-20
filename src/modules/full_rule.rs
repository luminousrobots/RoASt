use serde::{Deserialize, Serialize};

use crate::modules::view::display_view;

use super::{direction::Direction, view::View};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FullRule {
    pub view: Vec<Vec<char>>,
    pub direction: Direction,
    pub color: char,
}

pub fn print_full_rule(rule: &FullRule) {
    // println!("**********************");
    println!("View:");
    print_view(&rule.view);

    println!("Direction:{:?}", rule.direction);
    println!("Color:{:?}", rule.color);
    println!();
}

// Function to print the view (matrix) centered
fn print_view(view: &Vec<Vec<char>>) {
    // Find the maximum row length
    let max_row_length = view.iter().map(|row| row.len()).max().unwrap_or(0);

    for row in view {
        // Calculate padding spaces for centering
        let padding = (max_row_length - row.len()) / 2; // Use the exact difference for centering
        let spaces = "     ".repeat(padding); // Repeat space to center

        // Print the centered row
        println!("{}{:?}", spaces, row);
    }
}
