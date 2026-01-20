use serde_json;
use std::error::Error;
use std::fs;

use crate::modules::full_rule::print_full_rule;
use crate::modules::rule::print_rule;

use super::algorithm::Algorithm;
use super::position::Position;
use super::rule::Rule;
use super::view::{self, View};

pub struct AlgorithmManager {
    pub algorithms: Vec<Algorithm>,
}

impl AlgorithmManager {
    // Getter: Retrieves a reference to the algorithms list
    pub fn get_algorithms(&self) -> &[Algorithm] {
        &self.algorithms // Returns a reference to avoid cloning
    }

    // Setter: Adds a new algorithm
    pub fn add_algorithm(&mut self, algorithm: Algorithm) {
        self.algorithms.push(algorithm);
    }
    // Constructor: Loads algorithms from JSON file
    pub fn new(path: &str) -> Result<Self, Box<dyn Error>> {
        let filename = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), path);
        let data = fs::read_to_string(filename)?; // Optimized file reading
        let algorithms: Vec<Algorithm> = serde_json::from_str(&data)?;

        println!("Algorithms loaded successfully.");
        Ok(Self { algorithms })
    }

    // Retrieves an algorithm by index
    pub fn get_algorithm(&self, index: usize) -> Option<&Algorithm> {
        self.algorithms.get(index)
    }

    // Updates an existing algorithm
    pub fn set_algorithm(
        &mut self,
        index: usize,
        algorithm: Algorithm,
    ) -> Result<(), &'static str> {
        if index < self.algorithms.len() {
            self.algorithms[index] = algorithm;
            Ok(())
        } else {
            Err("Index out of bounds")
        }
    }

    // Prints loaded algorithms and their rules
    pub fn simple_load(&self) {
        println!("Loaded {} algorithms successfully!", self.algorithms.len());
        /*  for (i, algo) in self.algorithms.iter().enumerate() {
            for (j, rule) in algo.rules.iter().enumerate() {
                println!("---------------------{}----------------------", j);
                print_full_rule(rule);
            }
        }*/
    }

    // Extracts views and rules from a specific algorithm
    pub fn extract_views_and_rules(&self, index: usize) -> Option<(Vec<View>, Vec<Rule>)> {
        let mut views = Vec::new();
        let mut rules = Vec::new();

        if let Some(algo) = self.get_algorithm(index) {
            for rule in &algo.rules {
                if Self::is_valid_full_view(&rule.view) {
                    let view_with_starts = Self::convert_to_view(&rule.view);

                    for temp_view in Self::strip_stars(view_with_starts) {
                        let temp_view_id = views.len();
                        views.push(Self::remove_dots(temp_view));
                        rules.push(Rule {
                            view_id: temp_view_id,
                            direction: rule.direction.clone(),
                            color: rule.color,
                        });
                    }
                }
            }

            return Some((views, rules));
        }

        None
    }
    fn remove_dots(views: Vec<Position>) -> Vec<Position> {
        views.into_iter().filter(|(c, _, _)| *c != '.').collect()
    }

    // Extracts views and rules from a specific algorithm
    pub fn extract_views_and_rules_on_space(&self, index: usize) -> Option<(Vec<View>, Vec<Rule>)> {
        let mut views: Vec<Vec<Position>> = Vec::new();
        let mut rules: Vec<Rule> = Vec::new();

        if let Some(algo) = self.get_algorithm(index) {
            for rule in &algo.rules {
                if Self::is_valid_full_view(&rule.view) {
                    let view_with_starts = Self::convert_to_view(&rule.view);
                    for temp_view in Self::strip_stars(view_with_starts) {
                        let has_wall = temp_view.iter().any(|&(c, _, _)| c == 'W');

                        if !has_wall {
                            let temp_view_id = views.len();
                            views.push(temp_view);

                            rules.push(Rule {
                                view_id: temp_view_id,
                                direction: rule.direction.clone(),
                                color: rule.color,
                            });
                        }
                    }
                }
            }

            return Some((views, rules));
        }

        None
    }

    // Verifies if the given view matches a valid diamond format
    pub fn is_valid_full_view(full_view: &[Vec<char>]) -> bool {
        let total_rows = full_view.len();

        // Ensure height is odd (it must be 2*v - 1)
        if total_rows == 0 || total_rows % 2 == 0 {
            return false;
        }

        let v = total_rows / 2;

        // Iterate over rows to verify their width
        for (i, row) in full_view.iter().enumerate() {
            let expected_width = if i <= v {
                2 * i + 1
            } else {
                2 * (total_rows - 1 - i) + 1
            };

            if row.len() != expected_width {
                return false;
            }
        }

        true
    }
    // Verifies if the given view matches a valid diamond format
    pub fn convert_to_view(full_view: &[Vec<char>]) -> Vec<Position> {
        let mut view: Vec<Position> = vec![];
        let mut zero_zero_position: Option<Position> = None;

        for (i, row) in full_view.iter().enumerate() {
            for (j, &item) in row.iter().enumerate() {
                if item != '.' {
                    let x: isize = (j as isize) - ((row.len() as isize) / 2); // Center x within the row
                    let y: isize = (full_view.len() as isize) / 2 - (i as isize); // Center y in the view

                    let position = (item, x as i16, y as i16);

                    if x == 0 && y == 0 {
                        zero_zero_position = Some(position); // Store the (0,0) position
                    } else {
                        view.push(position);
                    }
                }
            }
        }

        // Insert (0,0) at the start if it exists
        if let Some(zero_pos) = zero_zero_position {
            view.insert(0, zero_pos);
        }

        view
    }

    /*pub fn remove_dots(positions: Vec<Position>) -> Vec<Position> {
        positions.into_iter().filter(|&(c, _, _)| c != '.').collect()
    }*/

    /*pub fn strip_stars(positions: Vec<Position>) -> Vec<Vec<Position>> {
        let has_star = positions.iter().any(|&(c, _, _)| c == '*');

        if !has_star {
            return vec![positions];
        }

        let mut set1 = Vec::new();
        let mut set2 = Vec::new();

        for &(c, x, y) in &positions {
            if c == '*' {
                // In the first set, replace '*' with '.'
                //  set1.push(('.', x, y));
                // In the second set, replace '*' with 'W'
                set2.push(('W', x, y));
            } else {
                // Keep unchanged items in both sets
                set1.push((c, x, y));
                set2.push((c, x, y));
            }
        }

        vec![set1, set2]
    }*/
    pub fn strip_stars(positions: Vec<Position>) -> Vec<Vec<Position>> {
        let star_positions: Vec<usize> = positions
            .iter()
            .enumerate()
            .filter(|(_, &(c, _, _))| c == '*')
            .map(|(i, _)| i)
            .collect();

        let num_stars = star_positions.len();
        let mut results = Vec::new();

        // Generate all 2^num_stars combinations
        for mask in 0..(1 << num_stars) {
            let mut new_positions = positions.clone();
            for (i, &pos) in star_positions.iter().enumerate() {
                new_positions[pos].0 = if (mask & (1 << i)) == 0 { '.' } else { 'W' };
            }
            results.push(new_positions);
        }

        results
    }
}
