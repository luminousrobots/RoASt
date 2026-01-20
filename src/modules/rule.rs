use std::{process::exit, thread::panicking};

use serde::{Deserialize, Serialize};

use crate::{
    methodology::globals::get_views,
    modules::{opacity_validator::are_belong_to_same_opacity_group, view::display_view},
};

use super::{direction::Direction, view::View};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub view_id: usize,
    pub direction: Direction,
    pub color: char,
}

pub fn generate_rules(colors: &[char], views: &[View], skip: usize) -> Vec<Rule> {
    let mut rules = Vec::new();

    // List of all possible directions including Idle
    let directions = vec![
        Direction::Up,
        Direction::Right,
        Direction::Down,
        Direction::Left,
        Direction::Idle,
    ];

    // Iterate through each view
    for view_id in skip..views.len() {
        // For each view, generate rules for all directions
        for direction in &directions {
            // For each direction, generate rules for all colors
            for color in colors {
                if *color != views[view_id][0].0 || *direction != Direction::Idle {
                    rules.push(Rule {
                        view_id: view_id,
                        direction: *direction,
                        color: *color,
                    });
                }
            }
        }
    }

    rules
}

/*******************************************************
 *                                                     *
 *                   Display Logs                      *
 *                                                     *
 *******************************************************/
pub fn print_rule(rule: &Rule, views: &[View], visibility: &i16) {
    // println!("**********************");
    println!("View:");
    display_view(&views[rule.view_id], &visibility);
    println!("Direction:{:?}", rule.direction);
    println!("Color:{:?}", rule.color);
    println!();
}

pub fn illuminate_new_generated_equivalent_rules_by_opacity(
    original_rules: &[Rule],
    generated_rules: &mut Vec<Rule>,
    views_grouped_by_opacity: &[Vec<usize>],
) {
    generated_rules.retain(|gen_rule| {
        // Keep gen_rule unless it matches some original_rule
        !original_rules.iter().any(|rule| {
            if rule.view_id == gen_rule.view_id {
                panic!(
                    "Error: rule and generated rule have the same view_id {}, they should be different",
                    rule.view_id
                );
            }
           let are_belong_to_same_opacity_group =
                are_belong_to_same_opacity_group(rule.view_id, gen_rule.view_id, views_grouped_by_opacity)
                && (rule.direction != gen_rule.direction
                || rule.color != gen_rule.color);
              /*  if are_belong_to_same_opacity_group {
                    println!("-------------------A rule is removed due to opacity equivalence--------------------");
                    println!("Original Rule:");
                    display_view(&get_views()[rule.view_id], &2);
                    println!("Direction:{:?}", rule.direction);
                    println!("Color:{:?}", rule.color);
                    println!();

                    println!("Generated Rule:");
                    display_view(&get_views()[gen_rule.view_id], &2);
                    println!("Direction:{:?}", gen_rule.direction);
                    println!("Color:{:?}", gen_rule.color);
                    println!();
                }*/
                are_belong_to_same_opacity_group
        })
    });
}
