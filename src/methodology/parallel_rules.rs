use crate::{
    methodology::{
        configuration::CONFIG,
        globals::{get_on_space_views, get_original_views_count},
        view::distribute_abstract_positions,
    },
    modules::{
        direction::{calculate_movement, rotate_direction, Direction},
        draft_rules::{self, DraftRule},
        parallel_rules::{
            adjust_positions, do_idle_views_conflict_with_original_rules,
            do_idle_views_conflict_with_rules, extract_starting_positions,
            get_suspected_rules_indexes, get_suspected_rules_indexes_from_robot_on_view,
            has_any_idle_robot_collision, has_rule_collision, is_idle_robot_at, ParallelRules,
        },
        position::{self, Position},
        rule::Rule,
        view::{
            are_equivalent_with_rotation, compare_views, get_view_from_positions, rotate_view, View,
        },
    },
};
use rayon::prelude::*;
use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

use super::globals::{get_number_of_robots, get_rules, get_views, get_visibility};

pub fn parallel_rules_generator() -> Vec<ParallelRules> {
    let rules = get_rules();
    let mut list_of_parallel_rules: Vec<_> = (0..rules.len())
        .into_par_iter()
        .map(|rule_index| {
            let draft_rules: Vec<DraftRule> = Vec::new();
            let mut local_rules = vec![];

            generate_parallel_rules(
                rule_index,
                0,
                // use a local vec instead of Arc<Mutex<_>>
                &mut local_rules,
                &draft_rules,
                &vec![],
                &vec![],
                0,
                0,
                &vec![],
            );

            local_rules
        })
        .flatten() // flatten Vec<Vec<ParallelRules>> into Vec<ParallelRules>
        .collect();

    // Deduplicate
    let mut unique_parallel_rules = vec![];

    'outer: for rule in list_of_parallel_rules {
        for existing in &unique_parallel_rules {
            if are_parallel_rules_identical(&rule, existing) {
                continue 'outer;
            }
        }
        unique_parallel_rules.push(rule);
    }

    unique_parallel_rules
}

fn are_parallel_rules_identical(a: &ParallelRules, b: &ParallelRules) -> bool {
    if a.rules.len() != b.rules.len() {
        return false;
    }

    let mut a_keys = a.rules.iter().map(|r| r.0).collect::<Vec<_>>();
    let mut b_keys = b.rules.iter().map(|r| r.0).collect::<Vec<_>>();

    // Sort so order does not matter when comparing
    a_keys.sort_unstable();
    b_keys.sort_unstable();

    // Now both sorted vectors can be compared directly
    a_keys == b_keys
}

pub fn generate_parallel_rules(
    rule_index: usize,
    suspected_rules_start_index: usize,
    list_of_parallel_rules: &mut Vec<ParallelRules>,
    draft_rules: &Vec<DraftRule>,
    movable_idle_robots: &Vec<Position>,
    fixed_idle_robots: &Vec<Position>,
    x: i16,
    y: i16,
    required_positions: &[Position],
) {
    if draft_rules.len() == *get_number_of_robots() {
        return;
    }

    if let Some(list_of_result) = place_on_subgrid(
        &rule_index,
        &draft_rules,
        &x,
        &y,
        &movable_idle_robots,
        &fixed_idle_robots,
        required_positions,
    ) {
        for (new_movable_idle_robots, new_fixed_idle_robots, x, y, _x, _y, angle) in list_of_result
        {
            let mut new_draft_rules = draft_rules.clone();
            new_draft_rules.push((rule_index, x, y, _x, _y));
            let (active_color_count, active_movement_count) =
                calculate_activation_counts(&new_draft_rules);

            //idle_robots_views will be calculated in can_rules_apply_globally
            let mut new_parallel_rules = ParallelRules {
                rules: new_draft_rules.clone(),
                movable_idle_robots: new_movable_idle_robots.clone(),
                fixed_idle_robots: new_fixed_idle_robots.clone(),
                idle_robots_views: vec![],
                active_color_count: active_color_count,
                active_movement_count: active_movement_count,
            };
            if !has_any_idle_robot_collision(&new_parallel_rules) {
                let idle_robots_views = calculate_idle_robots_views(
                    &new_parallel_rules,
                    &get_visibility(),
                    &get_views(),
                    &get_rules(),
                );
                new_parallel_rules.idle_robots_views = idle_robots_views;

                if !do_idle_views_conflict_with_rules(
                    &new_parallel_rules.idle_robots_views,
                    &new_parallel_rules.rules,
                    &get_rules(),
                    &get_views(),
                    &get_visibility(),
                    CONFIG.opacity,
                ) {
                    if !do_idle_views_conflict_with_original_rules(
                        &new_parallel_rules.idle_robots_views,
                        get_original_views_count(),
                        &get_views(),
                        &get_visibility(),
                        CONFIG.opacity,
                    ) {
                        if !has_isolated_robot(new_parallel_rules.clone()) {
                            list_of_parallel_rules.push(new_parallel_rules);
                        }
                    }
                }
                // }
            }
            if let Some((robot_marker, __x, __y)) = new_movable_idle_robots.get(0) {
                let rest_movable_idle_robots = &new_movable_idle_robots[1..]; // Use slicing to avoid cloning

                let suspected_rules_indexes = get_suspected_rules_indexes(
                    &robot_marker,
                    &suspected_rules_start_index,
                    &new_draft_rules,
                    &get_views(),
                    &get_rules(),
                );

                for suspected_rules_index in suspected_rules_indexes {
                    generate_parallel_rules(
                        suspected_rules_index,
                        suspected_rules_start_index,
                        list_of_parallel_rules,
                        &new_draft_rules,
                        &rest_movable_idle_robots.to_vec(), // Convert slice to Vec if needed
                        &new_fixed_idle_robots,
                        *__x, // Pass new X-coordinate
                        *__y, // Pass new Y-coordinate
                        &vec![],
                    );
                }

                // situation 2

                // Add the first robot to fixed_idle_robots and execute for the next idle robot if it exists
                let mut updated_fixed_idle_robots = new_fixed_idle_robots.clone();
                updated_fixed_idle_robots.push((*robot_marker, *__x, *__y));

                // Fix the first robot and search if there is a compatible view

                let positions = get_positions_near_idle_robots(
                    &(*robot_marker, *__x, *__y),
                    &get_visibility(),
                    &new_draft_rules, //:)
                    &rest_movable_idle_robots,
                    &updated_fixed_idle_robots,
                );

                let next_suspected_rules_indexes = get_suspected_rules_indexes_from_robot_on_view(
                    &robot_marker,
                    &suspected_rules_start_index,
                    &new_draft_rules,
                    &get_views(),
                    &get_rules(),
                );

                for position in positions.iter() {
                    for suspected_rules_index in next_suspected_rules_indexes.iter() {
                        generate_parallel_rules(
                            *suspected_rules_index,
                            suspected_rules_start_index,
                            list_of_parallel_rules,
                            &new_draft_rules,
                            &rest_movable_idle_robots.to_vec(),
                            &updated_fixed_idle_robots,
                            position.1, // Pass new X-coordinate
                            position.2, // Pass new Y-coordinate
                            &vec![(*robot_marker, *__x, *__y)],
                        );
                    }
                }

                // fix the 1 robot as idle and continue the process
                if !rest_movable_idle_robots.is_empty() {
                    if let Some((next_robot_marker, next_x, next_y)) =
                        rest_movable_idle_robots.get(0)
                    {
                        let next_rest_movable_idle_robots = &rest_movable_idle_robots[1..];

                        let next_suspected_rules_indexes = get_suspected_rules_indexes(
                            next_robot_marker,
                            &suspected_rules_start_index,
                            &new_draft_rules,
                            &get_views(),
                            &get_rules(),
                        );

                        for next_suspected_rules_index in next_suspected_rules_indexes {
                            generate_parallel_rules(
                                next_suspected_rules_index,
                                suspected_rules_start_index,
                                list_of_parallel_rules,
                                &new_draft_rules,
                                &next_rest_movable_idle_robots.to_vec(),
                                &updated_fixed_idle_robots,
                                *next_x,
                                *next_y,
                                &vec![],
                            );
                        }
                    }
                }
            }
        }
    } else {
        // print!(":D");
    }
}

fn get_positions_near_idle_robots(
    idle_robot: &Position,
    visibility: &i16,
    draft_rules: &[DraftRule],
    movable_robots: &[Position],
    fixed_robots: &[Position],
) -> Vec<Position> {
    let mut valid_positions = Vec::new();

    // Iterate over all possible positions within the visibility range
    for dx in -*visibility..=*visibility {
        for dy in -*visibility..=*visibility {
            if dx.abs() + dy.abs() <= *visibility {
                let candidate_position = (idle_robot.0, idle_robot.1 + dx, idle_robot.2 + dy);

                // Check if the candidate position is far from other robots in draft_rules
                let is_far_from_draft_rules = draft_rules.iter().all(|rule| {
                    let distance = (rule.1 - candidate_position.1).abs()
                        + (rule.2 - candidate_position.2).abs();
                    distance > *visibility
                });

                // Check if the candidate x,y position is not occupied by any robot in movable or fixed robots
                let is_not_in_movable_or_fixed = !movable_robots.iter().any(|robot| {
                    robot.1 == candidate_position.1 && robot.2 == candidate_position.2
                }) && !fixed_robots.iter().any(|robot| {
                    robot.1 == candidate_position.1 && robot.2 == candidate_position.2
                });

                if is_far_from_draft_rules && is_not_in_movable_or_fixed {
                    valid_positions.push(candidate_position);
                }
            }
        }
    }

    valid_positions
}

pub fn place_on_subgrid(
    rule_index: &usize,
    draft_rules: &Vec<DraftRule>,
    x: &i16,
    y: &i16,
    movable_idle_robots: &Vec<Position>,
    fixed_idle_robots: &Vec<Position>,
    required_positions: &[Position],
) -> Option<Vec<(Vec<Position>, Vec<Position>, i16, i16, i16, i16, i16)>> {
    let mut results = Vec::new();
    let view = &get_views()[get_rules()[*rule_index].view_id];
    let direction = &get_rules()[*rule_index].direction;

    if draft_rules.is_empty() {
        if let Some((new_movable_idle_robots, new_fixed_idle_robots)) = apply_view(
            &view,
            &0,
            &0,
            &movable_idle_robots,
            &fixed_idle_robots,
            &draft_rules,
            required_positions,
        ) {
            let (_x, _y) = calculate_movement(&direction, &x, &y);
            if !is_idle_robot_at(&new_fixed_idle_robots, &_x, &_y) {
                results.push((
                    new_movable_idle_robots,
                    new_fixed_idle_robots,
                    *x,
                    *y,
                    _x,
                    _y,
                    0,
                ));
            }
        }
    } else {
        // return  Some(results);
        let rotations_angles = vec![0, 90, 180, 270];

        for &angle in rotations_angles.iter() {
            let rotated_view = rotate_view(&view, angle);
            let rotated_direction = rotate_direction(&direction, angle);

            if let Some((new_movable_idle_robots, new_fixed_idle_robots)) = apply_view(
                &rotated_view,
                &x,
                &y,
                &movable_idle_robots,
                &fixed_idle_robots,
                &draft_rules,
                required_positions,
            ) {
                let (_x, _y) = calculate_movement(&rotated_direction, &x, &y);

                if !has_rule_collision(&draft_rules, &x, &y, &_x, &_y)
                    && !is_idle_robot_at(&new_fixed_idle_robots, &_x, &_y)
                {
                    results.push((
                        new_movable_idle_robots,
                        new_fixed_idle_robots,
                        *x,
                        *y,
                        _x,
                        _y,
                        angle,
                    ));
                }
            }
        }
    }
    if results.is_empty() {
        None
    } else {
        Some(results)
    }
}

fn apply_view(
    view: &View,
    x: &i16,
    y: &i16,
    movable_idle_robots: &View,
    fixed_idle_robots: &View,
    draft_rules: &Vec<DraftRule>,
    required_positions: &[Position],
) -> Option<(View, View)> {
    if view.len() <= 1 {
        return None;
    }

    let mut updated_movable_idle_robots = movable_idle_robots.clone();
    let mut updated_fixed_idle_robots = fixed_idle_robots.clone();
    let mut updated_view: View = vec![];

    let mut unmatched_fixed_idle_robots: HashSet<_> = fixed_idle_robots.iter().collect();
    let mut unmatched_movable_idle_robots: HashSet<_> = movable_idle_robots.iter().collect();
    let mut unmatched_draft_rules: HashSet<_> = draft_rules.iter().collect();

    if draft_rules.is_empty() {
        for pos in view.iter().skip(1) {
            if pos.0 == CONFIG.obstacle {
                let count_o = updated_fixed_idle_robots
                    .iter()
                    .filter(|&&p| p.0 == CONFIG.obstacle)
                    .count();
                if count_o != 0 {
                    return None;
                } else {
                    updated_fixed_idle_robots.push(*pos);
                }
            } else {
                updated_movable_idle_robots.push(*pos);
            }
        }
    } else {
        updated_view = view
            .iter()
            //.skip(1)
            .map(|&(c, vx, vy)| (c, vx + x, vy + y))
            .collect();

        let updated_entries: HashSet<(char, i16, i16)> = updated_view.iter().cloned().collect();

        for required in required_positions.iter() {
            if !updated_entries.contains(required) {
                return None; // A required (char, x, y) is missing
            }
        }

        let mut i = 1;
        'while_loop: while i < updated_view.len() {
            let position = updated_view[i];

            // Check against fixed robots
            for idle_robot in fixed_idle_robots {
                match compare_views(&position, idle_robot) {
                    None => return None,
                    Some(true) => {
                        unmatched_fixed_idle_robots.remove(&idle_robot);
                        i += 1;
                        continue 'while_loop;
                    }
                    Some(false) => {}
                }
            }
            if position.0 == CONFIG.obstacle {
                let count_o = updated_fixed_idle_robots
                    .iter()
                    .filter(|&&p| p.0 == CONFIG.obstacle)
                    .count();
                if count_o != 0 {
                    return None;
                } else {
                    for draft_rule in draft_rules {
                        let a = draft_rule.1 - position.1;
                        let b = draft_rule.2 - position.2;
                        if a.abs() + b.abs() <= *get_visibility() {
                            return None;
                        }
                    }
                    updated_fixed_idle_robots.push(position);
                    i += 1;
                    continue 'while_loop;
                }
            }

            // Check against movable robots
            for idle_robot in movable_idle_robots {
                match compare_views(&position, idle_robot) {
                    None => return None,
                    Some(true) => {
                        unmatched_movable_idle_robots.remove(&idle_robot);
                        i += 1;
                        continue 'while_loop;
                    }
                    Some(false) => {}
                }
            }

            // Check against draft rules
            for draft_rule in draft_rules {
                let rule_pos = (
                    get_views()[get_rules()[draft_rule.0].view_id][0].0,
                    draft_rule.1,
                    draft_rule.2,
                );
                match compare_views(&position, &rule_pos) {
                    None => return None,
                    Some(true) => {
                        unmatched_draft_rules.remove(draft_rule);
                        i += 1;
                        continue 'while_loop;
                    }
                    Some(false) => {}
                }
            }

            for draft_rule in draft_rules {
                let a = draft_rule.1 - position.1;
                let b = draft_rule.2 - position.2;
                if a.abs() + b.abs() <= *get_visibility() {
                    return None;
                }
            }
            updated_movable_idle_robots.push(position);

            i += 1; // Increment only if not removed
        }
    }

    for robot in unmatched_fixed_idle_robots {
        let a = robot.1 - updated_view[0].1;
        let b = robot.2 - updated_view[0].2;
        if a.abs() + b.abs() <= *get_visibility() {
            return None;
        }
    }
    for robot in unmatched_movable_idle_robots {
        let a = robot.1 - updated_view[0].1;
        let b = robot.2 - updated_view[0].2;
        if a.abs() + b.abs() <= *get_visibility() {
            return None;
        }
    }
    for robot in unmatched_draft_rules {
        let a = robot.1 - updated_view[0].1;
        let b = robot.2 - updated_view[0].2;
        if a.abs() + b.abs() <= *get_visibility() {
            return None;
        }
    }

    let count_fixed_robot = updated_fixed_idle_robots
        .iter()
        .filter(|&&p| p.0 != CONFIG.obstacle)
        .count();
    // Ensure we don't exceed the robot limit
    if updated_movable_idle_robots.len() + count_fixed_robot + draft_rules.len() + 1
        > *get_number_of_robots()
    {
        return None;
    }

    Some((
        updated_movable_idle_robots,
        updated_fixed_idle_robots.clone(),
    ))
}
/*
pub fn check_idle_views_against_rules(
    parallel_rules: &mut ParallelRules,
    visibility: &i16,
    views: &[View],
    rules: &[Rule],
) -> bool {
    let idle_robots_views = calculate_idle_robots_views(parallel_rules, visibility, views, rules);

    for idle_robot_view in idle_robots_views.iter() {
        for (rule_index, _, _, _, _) in &parallel_rules.rules {
            let applied_view = views[rules[*rule_index].view_id].clone();
            if are_equivalent_with_rotation(&idle_robot_view, &applied_view) {
                return false;
            }
        }

        for on_space_view in get_on_space_views().iter() {
            if are_equivalent_with_rotation(&idle_robot_view, &on_space_view) {
                return false;
            }
        }

        if CONFIG.opacity {
            let mut idle_robot_view_with_opacity = idle_robot_view.clone();
            distribute_abstract_positions(&mut idle_robot_view_with_opacity, *visibility, 0);
            for on_space_view in get_on_space_views().iter() {
                let mut on_space_view_with_opacity = on_space_view.clone();
                distribute_abstract_positions(&mut on_space_view_with_opacity, *visibility, 0);
                if are_equivalent_with_rotation(
                    &idle_robot_view_with_opacity,
                    &on_space_view_with_opacity,
                ) {
                    return false;
                }
            }
            //forgot opacity for applied rules views
            for (rule_index, _, _, _, _) in &parallel_rules.rules {
                let mut applied_view = views[rules[*rule_index].view_id].clone();
                distribute_abstract_positions(&mut applied_view, *visibility, 0);
                if are_equivalent_with_rotation(&idle_robot_view_with_opacity, &applied_view) {
                    return false;
                }
            }
        }
    }

    parallel_rules.idle_robots_views = idle_robots_views;
    true
}
*/
fn calculate_idle_robots_views(
    parallel_rules: &ParallelRules,
    visibility: &i16,
    views: &[View],
    rules: &[Rule],
) -> Vec<View> {
    let mut idle_robots_views: Vec<View> = Vec::new();
    let (initial_positions, idle_robot_count) =
        extract_starting_positions(parallel_rules, views, rules);

    for index in 0..idle_robot_count {
        let positions = adjust_positions(index, &initial_positions);
        let view = get_view_from_positions(&index, &positions, *visibility);
        idle_robots_views.push(view.clone());
    }

    idle_robots_views
}

pub fn calculate_activation_counts(draft_rules: &Vec<DraftRule>) -> (usize, usize) {
    let rules = get_rules();
    let views = get_views();

    let mut color_set = 0;
    let mut movement_set = 0;

    for (rule_index, _, _, _, _) in draft_rules {
        let rule = &rules[*rule_index];
        if rule.direction != Direction::Idle {
            movement_set += 1;
        }
        if rule.color != views[rule.view_id][0].0 {
            color_set += 1;
        }
    }

    (color_set, movement_set)
}

fn has_isolated_robot(parallel_rules: ParallelRules) -> bool {
    for parallel_rule in parallel_rules.rules.iter() {
        let mut view = get_views()[get_rules()[parallel_rule.0].view_id].clone();

        if CONFIG.opacity {
            distribute_abstract_positions(&mut view, *get_visibility());
        }

        let has_other_robot = view
            .iter()
            .skip(1)
            .any(|pos| pos.0 != CONFIG.obstacle && pos.0 != 'X');

        if !has_other_robot {
            return true; // Found an isolated robot
        }
    }
    false
}
