use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::{
    methodology::globals::{get_rules, get_views},
    methodology::view::{apply_abstract_positions_distribution, distribute_abstract_positions},
    modules::{
        opacity_validator::are_conflicting_by_opacity,
        rule::print_rule,
        simulator::extract_dualpositions,
        view::{self, View},
    },
};

use super::{
    draft_rules::DraftRule,
    grid::generate_grids,
    position::{self, Position},
    rule::Rule,
    view::{are_equivalent_with_rotation, get_view_from_positions},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelRules {
    pub rules: Vec<DraftRule>,
    pub movable_idle_robots: Vec<Position>,
    pub fixed_idle_robots: Vec<Position>,
    pub idle_robots_views: Vec<View>,
    pub active_color_count: usize, // Number of color change activations
    pub active_movement_count: usize,
}

pub fn is_idle_robot_at(idle_robots: &Vec<Position>, _x: &i16, _y: &i16) -> bool {
    for idle_robot in idle_robots {
        if idle_robot.1 == *_x && idle_robot.2 == *_y {
            return true;
        }
    }
    false
}
pub fn has_any_idle_robot_collision(parallel_rules: &ParallelRules) -> bool {
    for (_, _, _, x, y) in &parallel_rules.rules {
        if is_idle_robot_at(&parallel_rules.fixed_idle_robots, x, y) {
            return true;
        }
        if is_idle_robot_at(&parallel_rules.movable_idle_robots, x, y) {
            return true;
        }
    }
    false
}

pub fn has_rule_collision(
    draft_rules: &Vec<DraftRule>,
    x: &i16,
    y: &i16,
    _x: &i16,
    _y: &i16,
) -> bool {
    for draft_rule in draft_rules {
        let x2 = draft_rule.1;
        let y2 = draft_rule.2;
        let _x2 = draft_rule.3;
        let _y2 = draft_rule.4;

        if _x2 == *_x && _y2 == *_y {
            return true;
        }

        if _x2 == *x && _y2 == *y && x2 == *_x && y2 == *_y {
            return true;
        }
    }
    false
}
pub fn extract_starting_positions(
    parallel_rules: &ParallelRules,
    views: &[View],
    rules: &[Rule],
) -> (Vec<Position>, usize) {
    let mut initial_positions: Vec<Position> = Vec::new();

    // Add fixed and movable idle robots
    initial_positions.extend(&parallel_rules.fixed_idle_robots);
    initial_positions.extend(&parallel_rules.movable_idle_robots);

    // Save the length after adding idle robots
    let idle_robot_count = initial_positions.len();

    // Add robots based on rules
    for rule_info in &parallel_rules.rules {
        initial_positions.push((
            views[rules[rule_info.0].view_id][0].0,
            rule_info.1,
            rule_info.2,
        ));
    }

    // Return adjusted positions along with the count of idle robots
    (initial_positions, idle_robot_count)
}

pub fn extract_ending_positions(
    parallel_rules: &ParallelRules,
    rules: &[Rule],
) -> (Vec<Position>, usize) {
    let mut initial_positions: Vec<Position> = Vec::new();

    // Add fixed and movable idle robots
    initial_positions.extend(&parallel_rules.fixed_idle_robots);
    initial_positions.extend(&parallel_rules.movable_idle_robots);

    // Save the length after adding idle robots
    let idle_robot_count = initial_positions.len();

    // Add robots based on rules
    for rule_info in &parallel_rules.rules {
        initial_positions.push((rules[rule_info.0].color, rule_info.3, rule_info.4));
    }

    // Return adjusted positions along with the count of idle robots
    (initial_positions, idle_robot_count)
}

pub fn adjust_positions(reference_index: usize, positions: &[Position]) -> Vec<Position> {
    assert!(
        positions.len() >= 2,
        "The positions vector must contain at least two elements."
    );
    assert!(
        reference_index < positions.len(),
        "The reference index is out of bounds."
    );

    let (shift_x, shift_y) = (positions[reference_index].1, positions[reference_index].2);

    positions
        .iter()
        .map(|&(ch, x, y)| (ch, x - shift_x, y - shift_y))
        .collect()
}

pub fn get_suspected_rules_indexes(
    robot_marker: &char,
    rule_index: &usize,
    draft_rules: &[DraftRule],
    views: &[View],
    rules: &[Rule],
) -> Vec<usize> {
    rules
        .iter()
        .enumerate()
        .skip(*rule_index) // Skip rules up to the given index
        .filter_map(|(index, rule)| {
            // Check if the robot_marker is at the center of the rule's view
            if views[rule.view_id][0].0 != *robot_marker {
                return None;
            }

            for &(rule_id, _, _, _, _) in draft_rules {
                if are_equivalent_with_rotation(
                    &views[rules[rule_id].view_id],
                    &views[rule.view_id],
                ) && (rules[rule_id].direction != rule.direction
                    || rules[rule_id].color != rule.color)
                {
                    return None; // Skip if direction or color is different
                }
            }

            Some(index) // Only return the index
        })
        .collect()
}

pub fn get_suspected_rules_indexes_from_robot_on_view(
    robot_marker: &char,
    rule_index: &usize, // Changed from start_rule_index to match user's provided code
    draft_rules: &[DraftRule],
    views: &[View],
    rules: &[Rule],
) -> Vec<usize> {
    rules
        .iter()
        .enumerate()
        .skip(*rule_index)
        .filter_map(|(index, rule)| {
            let rule_view = &views[rule.view_id];

            if rule_view.len() <= 1 || !rule_view.iter().skip(1).any(|pos| pos.0 == *robot_marker) {
                return None;
            }

            if draft_rules.iter().any(|&(rule_id, _, _, _, _)| {
                // Assuming rule_id from draft_rules is a valid index for `rules`.
                // The user-provided code for optimization does not have an explicit bounds check here.
                are_equivalent_with_rotation(&views[rules[rule_id].view_id], rule_view)
                    && (rules[rule_id].direction != rule.direction
                        || rules[rule_id].color != rule.color)
            }) {
                return None;
            }

            Some(index)
        })
        .collect()
}

/*
pub fn adjust_positions_to_2d(
    reference_index: usize,
    positions: &Vec<Position>,
) -> Vec<Position> {
    if positions.len() < 2 {
        panic!("The positions vector must contain at least two elements.");
    }

    if reference_index >= positions.len() {
        panic!("The reference index is out of bounds.");
    }

    let mut temp_positions: Vec<Position> = vec![];

    // Extract shift values from the reference position
    let (shift_x, shift_y) = (
        positions[reference_index].1 ,
        positions[reference_index].2 ,
    );

    // Modify positions based on the shift
    for &(ch, x, y) in positions {
        let new_x = x - shift_x;
        let new_y = y - shift_y;
        temp_positions.push((ch, new_x, new_y));
    }

    temp_positions
}*/

/*******************************************************
 *                                                     *
 *                   Display Logs                      *
 *                                                     *
 *******************************************************/

pub fn print_parallel_rules(
    list_of_parallel_rules: &Vec<ParallelRules>,
    visibility: &i16,
    number_of_robots: &usize,
    views: &[View],
    rules: &[Rule],
) {
    for (index, parallel_rules) in list_of_parallel_rules.iter().enumerate() {
        println!();
        println!(
            "------------------------ParallelRule {} ({} rules)------------------------",
            index,
            parallel_rules.rules.len()
        );

        // Print all information for the current parallel rule
        print_parallel_rule_info(
            &parallel_rules,
            &visibility,
            &number_of_robots,
            &views,
            &rules,
        );
    }
}

pub fn print_parallel_rule_info(
    parallel_rules: &ParallelRules,
    visibility: &i16,
    number_of_robots: &usize,
    views: &[View],
    rules: &[Rule],
) {
    // Print rule details
    for (rule_index, x, y, _x, _y) in &parallel_rules.rules {
        println!("------------------{rule_index}------------------");
        println!("({}, {}) to ({}, {})", x, y, _x, _y);
        println!("{:?}", views[rules[*rule_index].view_id]);

        print_rule(&rules[*rule_index], views, &visibility);
    }
    for (ch, x, y) in &parallel_rules.movable_idle_robots {
        println!("movable_idle_robots {} in ({}, {})", ch, x, y);
    }
    for (ch, x, y) in &parallel_rules.fixed_idle_robots {
        println!("fixed_idle_robots {} in ({}, {})", ch, x, y);
    }
    println!(
        "Starting: {:?}",
        extract_starting_positions(&parallel_rules, views, rules)
    );
    println!(
        "Ending:   {:?}",
        extract_ending_positions(&parallel_rules, rules)
    );
    println!("active_color_count: {}", parallel_rules.active_color_count);
    println!(
        "active_movement_count: {}",
        parallel_rules.active_movement_count
    );
    // println!("{:?}", extract_dualpositions(&parallel_rules,views,rules));

    let (starting_positions, _) = extract_starting_positions(parallel_rules, views, rules);
    let (ending_positions, _) = extract_ending_positions(parallel_rules, rules);
    generate_grids(
        &starting_positions,
        &ending_positions,
        visibility,
        number_of_robots,
    );
}
pub fn check_parallel_rules_compatibility(
    a: &ParallelRules,
    b: &ParallelRules,
    rules: &[Rule],
    opacity: bool,
    views: &[View],
    visibility: &i16,
) -> bool {
    if !check_rule_level_conflicts(a, b, rules) {
        return false;
    }

    // check idle views against A
    if do_idle_views_conflict_with_rules(
        &b.idle_robots_views.clone(),
        &a.rules,
        rules,
        views,
        visibility,
        opacity,
    ) {
        return false;
    }

    // check idle views against B
    if do_idle_views_conflict_with_rules(
        &a.idle_robots_views.clone(),
        &b.rules,
        rules,
        views,
        visibility,
        opacity,
    ) {
        return false;
    }

    true
}

/// Checks if there are direct conflicts between rule definitions
fn check_rule_level_conflicts(
    a: &ParallelRules,
    b: &ParallelRules,
    rules: &[Rule],
) -> bool {
    for (i, _, _, _, _) in &a.rules {
        for (j, _, _, _, _) in &b.rules {
            // Same view_id but different behavior → conflict
            if rules[*i].view_id == rules[*j].view_id
                && (rules[*i].direction != rules[*j].direction
                    || rules[*i].color != rules[*j].color)
            {
                return false;
            }

        }
    }
    true
}

/// Checks if a set of idle views conflict with the given rules
pub fn do_idle_views_conflict_with_rules(
    idle_views: &[View],
    draft_rules: &Vec<DraftRule>,
    rules: &[Rule],
    views: &[View],
    visibility: &i16,
    opacity: bool,
) -> bool {
    for idle_view in idle_views {
        // check directly against rules
        for (rule_index, _, _, _, _) in draft_rules {
            let applied_view = views[rules[*rule_index].view_id].clone();
            if !opacity {
                if are_equivalent_with_rotation(idle_view, &applied_view) {
                    return true;
                }
            } else {
                let mut idle_view_with_opacity = idle_view.clone();
                distribute_abstract_positions(&mut idle_view_with_opacity, *visibility);

                let mut applied_view_with_opacity = views[rules[*rule_index].view_id].clone();
                distribute_abstract_positions(&mut applied_view_with_opacity, *visibility);

                if are_equivalent_with_rotation(&idle_view_with_opacity, &applied_view_with_opacity)
                {
                    return true;
                }
            }
        }
    }
    false
}

pub fn do_idle_views_conflict_with_original_rules(
    idle_views: &[View],
    original_views_length: usize,
    views: &[View],
    visibility: &i16,
    opacity: bool,
) -> bool {
    for idle_view in idle_views {
        // check directly against rules
        for i in 0..original_views_length {
            if !opacity {
                if are_equivalent_with_rotation(idle_view, &views[i]) {
                    return true;
                }
            } else {
                let mut idle_view_with_opacity = idle_view.clone();
                distribute_abstract_positions(&mut idle_view_with_opacity, *visibility);

                let mut original_view_with_opacity = views[i].clone();
                distribute_abstract_positions(&mut original_view_with_opacity, *visibility);

                if are_equivalent_with_rotation(
                    &idle_view_with_opacity,
                    &original_view_with_opacity,
                ) {
                    return true;
                }
            }
        }
    }
    false
}

pub fn extract_rules(
    parallel_rules_goal: &[usize],
    list_of_parallel_rules: &[ParallelRules],
) -> Vec<usize> {
    let mut rules = HashSet::new(); // Using HashSet to avoid duplicates

    for &parallel_rule_index in parallel_rules_goal {
        for parallel_rule in &list_of_parallel_rules[parallel_rule_index].rules {
            rules.insert(parallel_rule.0); // Insert into HashSet (duplicates will be ignored)
        }
    }
    let mut sorted_rules: Vec<usize> = rules.into_iter().collect();
    sorted_rules.sort(); // ← Sort in ascending order
    sorted_rules
}

/// Calculate total activation count (color + movement) for an execution
pub fn calculate_total_activation(
    execution: &[usize],
    list_of_parallel_rules: &[ParallelRules],
) -> usize {
    execution
        .iter()
        .map(|&rule_index| {
            let parallel_rule = &list_of_parallel_rules[rule_index];
            parallel_rule.active_color_count + parallel_rule.active_movement_count
        })
        .sum()
}

/// Calculate color activation count for an execution
pub fn calculate_color_activation(
    execution: &[usize],
    list_of_parallel_rules: &[ParallelRules],
) -> usize {
    execution
        .iter()
        .map(|&rule_index| {
            let parallel_rule = &list_of_parallel_rules[rule_index];
            parallel_rule.active_color_count
        })
        .sum()
}

/// Calculate movement activation count for an execution
pub fn calculate_movement_activation(
    execution: &[usize],
    list_of_parallel_rules: &[ParallelRules],
) -> usize {
    execution
        .iter()
        .map(|&rule_index| {
            let parallel_rule = &list_of_parallel_rules[rule_index];
            parallel_rule.active_movement_count
        })
        .sum()
}
