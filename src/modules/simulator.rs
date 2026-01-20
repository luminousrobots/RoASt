use std::{
    collections::HashSet,
    process::exit,
    sync::{Arc, Mutex},
};

use rayon::{
    iter::{IntoParallelIterator, ParallelIterator},
    vec,
};

use crate::{
    methodology::configuration::CONFIG,
    modules::{
        opacity_validator::{are_belong_to_same_opacity_group, are_conflicting_by_opacity},
        parallel_rules::check_parallel_rules_compatibility,
        simulation_config::Target,
    },
};

use super::{
    dual_position::DualPosition,
    parallel_rules::{
        adjust_positions, extract_ending_positions, extract_starting_positions, ParallelRules,
    },
    position::{rotate_point, Position},
    rule::{self, Rule},
    view::{are_equivalent, are_equivalent_with_rotation, View},
};

pub fn adjust_positions_dual_position(
    reference_index: usize,
    positions: &Vec<DualPosition>,
) -> Vec<DualPosition> {
    if positions.len() < 2 {
        panic!("The positions vector must contain at least two elements.");
    }

    if reference_index >= positions.len() {
        panic!("The reference index is out of bounds.");
    }

    let mut temp_positions: Vec<DualPosition> = vec![];

    // Extract shift values from the reference position
    let (shift_x, shift_y) = (positions[reference_index].1, positions[reference_index].2);

    // Modify positions based on the shift
    for &(r1, x, y, r2, new_x, new_y) in positions {
        let _x = x - shift_x;
        let _y = y - shift_y;
        let _new_x = new_x - shift_x;
        let _new_y = new_y - shift_y;
        temp_positions.push((r1, _x, _y, r2, _new_x, _new_y));
    }

    temp_positions
}
pub fn has_cycle(current_execution: &Vec<Position>, execution: &Vec<Vec<Position>>) -> bool {
    for ex in execution.iter() {
        if are_equivalent(&current_execution, &ex) {
            return true;
        }
    }
    false
}

pub fn is_parallel_rule_compatible_with_execution(
    execution: &[usize],
    current_parallel_rules_index: usize,
    list_of_parallel_rules: &[ParallelRules],
    rules: &[Rule],
    opacity: bool,
    views: &[View],
    visibility: &i16,
) -> bool {
    let current = &list_of_parallel_rules[current_parallel_rules_index];
    for &idx in execution {
        let executed = &list_of_parallel_rules[idx];
        if !check_parallel_rules_compatibility(
            current,
            executed,
            rules,
            opacity,
            views,
            visibility,
        ) {
            return false;
        }
    }
    true
}

pub fn extract_dualpositions(
    parallel_rules: &ParallelRules,
    views: &[View],
    rules: &[Rule],
) -> Vec<DualPosition> {
    // Initialize the vector for storing initial positions
    let mut positions: Vec<DualPosition> = vec![];

    // Add fixed idle robots to the initial positions (convert to the required type)
    positions.extend(
        parallel_rules
            .fixed_idle_robots
            .iter()
            .map(|&(c, x, y)| (c, x, y, c, x, y)),
    );

    // Add movable idle robots to the initial positions (convert to the required type)
    positions.extend(
        parallel_rules
            .movable_idle_robots
            .iter()
            .map(|&(c, x, y)| (c, x, y, c, x, y)),
    );

    // Add robots based on rules
    for rule_info in &parallel_rules.rules {
        let rule_index = rule_info.0;
        let x = rule_info.1;
        let y = rule_info.2;
        let new_x = rule_info.3;
        let new_y = rule_info.4;

        // Add the robot ID from the view of the rule
        positions.push((
            views[rules[rule_info.0].view_id][0].0,
            x,
            y,
            rules[rule_index].color,
            new_x,
            new_y,
        ));
    }
    positions
}
pub fn rotate_vector_dual_position(position: Vec<DualPosition>, angle: i16) -> Vec<DualPosition> {
    position
        .into_iter()
        .map(|(r1, x, y, r2, new_x, new_y)| {
            let (_x, _y) = rotate_point(&x, &y, &angle);
            let (_new_x, _new_y) = rotate_point(&new_x, &new_y, &angle);
            (r1, _x, _y, r2, _new_x, _new_y)
        })
        .collect()
}
pub fn search_for_solutions(
    initial_positions: Vec<Position>,
    list_of_starting_positions: &Vec<Vec<Position>>,
) -> Vec<usize> {
    list_of_starting_positions
        .iter()
        .enumerate()
        .filter_map(|(index, starting_positions)| {
            for reference_index in 0..initial_positions.len() {
                let initial_position_adj: Vec<Position> =
                    adjust_positions(reference_index, &initial_positions);
                if are_equivalent_with_rotation(&initial_position_adj, &starting_positions) {
                    return Some(index);
                }
            }
            None
        })
        .collect()
}

fn compute_next_positions(
    steps: usize,
    positions: &Vec<DualPosition>,
    initial_position_adjusted: &Vec<Position>,
    initial_position: &Vec<Position>,
    boundaries: Option<(i16, i16, i16, i16)>,
    exclusive_points: &[(i16, i16)],
    goal_position: &[Position],
) -> Option<Vec<Position>> {
    let mut next_position: Vec<Position> = Vec::new();
    let mut used_indices: HashSet<usize> = HashSet::new();

    // Check if the lengths of the vectors are the same
    if positions.len() != initial_position_adjusted.len() {
        return None;
    }
    if initial_position.len() < initial_position_adjusted.len() {
        panic!("ERROR 10: The initial positions here should be equal");
    }

    // Iterate through initial_position_adjusted and match with positions
    for (i, (r, x, y)) in initial_position_adjusted.iter().enumerate() {
        let mut found = false;

        for (idx, (r1, x1, y1, r2, x2, y2)) in positions.iter().enumerate() {
            if r == r1 && x == x1 && y == y1 {
                // Ensure this index is used only once
                if !used_indices.insert(idx) {
                    return None; // Duplicate match found
                }

                let new_x = initial_position[i].1 + (x2 - x1);
                let new_y = initial_position[i].2 + (y2 - y1);
                // Check if the new position is within boundaries (if provided)
                if let Some((bx1, bx2, by1, by2)) = boundaries {
                    if !is_position_inside_boundary_exclusive(new_x, new_y, bx1, bx2, by1, by2) {
                        return None;
                    }
                }

                // Check if the new position is in the exclusive points (if provided)
                if exclusive_points != vec![] {
                    if is_position_exclusive(new_x, new_y, exclusive_points) {
                        return None;
                    }
                }

                //process if this position not an obstacle
                if *r != CONFIG.obstacle {
                    //stop if this position dont have a chance to reach the goal
                    let mut min_distance = i16::MAX;
                    for (r_g, x_g, y_g) in goal_position {
                        if *r_g == CONFIG.obstacle {
                            continue;
                        }
                        let distance = (new_x - x_g).abs() + (new_y - y_g).abs();
                        if distance < min_distance {
                            min_distance = distance;
                        }
                    }
                    if min_distance > steps as i16 - 1 {
                        return None;
                    }
                }
                next_position.push((*r2, new_x, new_y));
                found = true;
                break;
            }
        }

        if !found {
            return None;
        }
    }

    if next_position.len() != initial_position_adjusted.len() {
        return None;
    }

    Some(next_position)
}
/*fn is_position_inside_boundary_exclusive(
    x: &i16,
    y: &i16,
    x1: &i16,
    x2: &i16,
    y1: &i16,
    y2: &i16,
) -> bool {
    x > x1 && x < x2 && y > y1 && y < y2
}*/
/*fn is_position_inside_boundary_exclusive(
    x: &i16,
    y: &i16,
    x1: &i16,
    x2: &i16,
    y1: &i16,
    y2: &i16,
) -> bool {
    x > x1
        && x < x2
        && y > y1
        && y < y2
        && !((*x == x1 + 1 && *y == y1 + 1)
            || (*x == x1 + 1 && *y == y2 - 1)
            || (*x == x2 - 1 && *y == y1 + 1)
            || (*x == x2 - 1 && *y == y2 - 1))
}*/
fn is_position_inside_boundary_exclusive(
    x: i16,
    y: i16,
    x1: i16,
    x2: i16,
    y1: i16,
    y2: i16,
) -> bool {
    // Ensure the point is within the boundary (exclusive)
    x > x1 && x < x2 && y > y1 && y < y2
}
fn is_position_exclusive(x: i16, y: i16, exclusive_points: &[(i16, i16)]) -> bool {
    exclusive_points.contains(&(x, y))
}

pub fn search_for_solutions_v1(
    steps: usize,
    list_of_positions: &Vec<Vec<DualPosition>>,
    initial_position_adjusted: &Vec<Position>,
    initial_position: &Vec<Position>,
    boundaries: Option<(i16, i16, i16, i16)>,
    exclusive_points: &[(i16, i16)],
    goal_position: &[Position],
) -> Vec<(usize, Vec<Position>)> {
    list_of_positions
        .iter()
        .enumerate()
        .filter_map(|(index, positions)| {
            for reference_index in 0..positions.len() {
                let new_positions_adjusted =
                    adjust_positions_dual_position(reference_index, &positions);

                for &degrees in &[0, 90, 180, 270] {
                    if let Some(next_positions) = compute_next_positions(
                        steps,
                        &rotate_vector_dual_position(new_positions_adjusted.clone(), degrees),
                        &initial_position_adjusted,
                        &initial_position,
                        boundaries,
                        exclusive_points,
                        goal_position,
                    ) {
                        return Some((index, next_positions));
                    }
                }
            }
            None
        })
        .collect()
}

/*******************************************************
 *                                                     *
 *           Method 1: Find All Executions            *
 *                                                     *
 *******************************************************/

pub fn simulate(
    steps: usize,
    execution: Vec<usize>,
    initial_position: Vec<Position>,
    all_executions: Arc<Mutex<Vec<Vec<usize>>>>,
    list_of_starting_positions: &Vec<Vec<Position>>,
    list_of_result_positions: &Vec<Vec<Position>>,
    list_of_parallel_rules: &Vec<ParallelRules>,
    rules: &[Rule],

    views: &[View],
    visibility: &i16,
) {
    if steps == 0 {
        let mut all_executions_lock = all_executions.lock().unwrap();
        all_executions_lock.push(execution);
        return;
    }

    let solutions = search_for_solutions(initial_position, &list_of_starting_positions);

    let _results: Vec<()> = solutions
        .into_par_iter()
        .map(|solution| {
            if !is_parallel_rule_compatible_with_execution(
                &execution,
                solution,
                &list_of_parallel_rules,
                &rules,
                false,
                views,
                visibility,
            ) {
                return;
            }
            let mut current_execution = execution.clone();
            current_execution.push(solution);

            // Determine the next set of positions
            if let Some(result_positions) = list_of_result_positions.get(solution) {
                simulate(
                    steps - 1,                   // Decrement steps
                    current_execution,           // New execution vector
                    result_positions.clone(),    // New initial positions
                    Arc::clone(&all_executions), // Pass mutable reference
                    list_of_starting_positions,
                    list_of_result_positions,
                    list_of_parallel_rules,
                    rules,
                    views,
                    visibility,
                );
            }
        })
        .collect();
}

/*******************************************************
 *                                                     *
 *                Method 2: Achieve Goal               *
 *                                                     *
 *******************************************************/

pub fn simulate_v2(
    steps: usize,
    execution: Vec<usize>,
    execution_position: Vec<Vec<Position>>,
    current_position: Vec<Position>,
    execution_history: Arc<Mutex<Vec<Vec<usize>>>>,
    execution_position_history: Arc<Mutex<Vec<Vec<Vec<Position>>>>>,
    list_of_positions: &Vec<Vec<DualPosition>>,
    list_of_parallel_rules: &Vec<ParallelRules>,
    goal_position: &[Position],
    boundaries: Option<(i16, i16, i16, i16)>,
    exclusive_points: &[(i16, i16)],
    waypoint_positions: &[(i16, i16)],
    rules: &[Rule],
    visibility: i16,
    opacity: bool,
    views: &[View],
) {
    if steps == 0 {
        return;
    }

    let (initial_position_adjusted, current_positions, isolated_positions) = {
        let (c_positions, c_isolated_positions) =
            separate_isolated_positions(&current_position, visibility);

        if c_positions.len() < 2 {
            return;
        }

        (
            adjust_positions(0, &c_positions),
            c_positions,
            c_isolated_positions,
        )
    };

    let mut solutions = search_for_solutions_v1(
        steps,
        list_of_positions,
        &initial_position_adjusted,
        &current_positions,
        boundaries,
        exclusive_points,
        goal_position,
    );

    // Append isolated positions if any
    if !isolated_positions.is_empty() {
        for (_, ref mut next_positions) in &mut solutions {
            next_positions.extend(isolated_positions.clone());
        }
    }

    /*  // Sort by index first, then by active count for ties
    solutions.sort_by(
        |(index_a, _, active_count_a), (index_b, _, active_count_b)| {
            index_a
                .cmp(index_b) // Primary: sort by index
                .then_with(|| active_count_a.cmp(active_count_b)) // Secondary: sort by active count
        },
    );*/
    // ✅ CORRECT: Sort only by index, keep the tuple intact
    // solutions.sort_by_key(|(index, _positions, _active_count)| *index);

    // 🔑 UNCOMMENT AND FIX: Add deterministic sorting
    /*   solutions.sort_by(
        |(index_a, positions_a), (index_b, positions_b)| {
            index_a
                .cmp(index_b) // Primary: sort by rule index
                .then_with(|| positions_a.cmp(positions_b)) // Secondary: sort by positions
        },
    );*/
    let _results: Vec<()> = solutions
        .into_par_iter()
        .map(|(exe, exe_pos)| {
            let exe_set: HashSet<(i16, i16)> = exe_pos.iter().map(|&(_, x, y)| (x, y)).collect();

            let remain_waypoint_positions: Vec<(i16, i16)> = waypoint_positions
                .iter()
                .cloned()
                .filter(|pos| !exe_set.contains(pos))
                .collect();

            if !is_parallel_rule_compatible_with_execution(
                &execution,
                exe,
                &list_of_parallel_rules,
                &rules,
                opacity,
                views,
                &visibility,
            ) || has_cycle(&exe_pos, &execution_position)
            {
                return;
            }

            let mut updated_execution = execution.clone();
            updated_execution.push(exe);

            let mut updated_execution_position = execution_position.clone();
            updated_execution_position.push(exe_pos.clone());

            if move_on_ligne_goal(&exe_pos, &goal_position.to_vec()) {
                if !remain_waypoint_positions.is_empty() {
                    return;
                }
                let mut history = execution_history.lock().unwrap();
                history.push(updated_execution.clone());

                let mut pos_history = execution_position_history.lock().unwrap();
                pos_history.push(updated_execution_position);
            } else {
                simulate_v2(
                    steps - 1,
                    updated_execution,
                    updated_execution_position,
                    exe_pos,
                    Arc::clone(&execution_history),
                    Arc::clone(&execution_position_history),
                    list_of_positions,
                    list_of_parallel_rules,
                    goal_position,
                    boundaries,
                    exclusive_points,
                    &remain_waypoint_positions,
                    rules,
                    visibility,
                    opacity,
                    views,
                );
            }
            //    }
        })
        .collect();
}

fn move_on_ligne_goal(after_execution: &Vec<Position>, goal_position: &Vec<Position>) -> bool {
    if are_equivalent(&after_execution, &goal_position) {
        return true;
    }

    false
}

pub fn simulation(
    goal_number: usize,
    initial_positions: &[Position],
    targets: &[Target],
    list_of_parallel_rules: &Vec<ParallelRules>,
    boundaries: Option<(i16, i16, i16, i16)>,
    views: &[View],
    rules: &[Rule],
    visibility: i16,
    opacity: bool,
) -> (Vec<Vec<Vec<usize>>>, Vec<Vec<Vec<Vec<Position>>>>) {
    println!("**************************************************");
    println!("*                                                *");
    println!(
        "*             Simulator V2  : goal {}             *",
        goal_number
    );
    println!("*                                                *");
    println!("**************************************************");
    println!();
    println!(
        "🚀 Running simulation with {} parallel rules...",
        list_of_parallel_rules.len()
    );
    println!();

    let mut executions_result = vec![];
    let mut execution_positions_result: Vec<Vec<Vec<Vec<Position>>>> = vec![];

    let list_of_positions: Vec<Vec<DualPosition>> = list_of_parallel_rules
        .iter()
        .map(|parallel_rules| extract_dualpositions(parallel_rules, &views, &rules))
        .collect();

    for (j, target) in targets.iter().enumerate() {
        let executions = Arc::new(Mutex::new(vec![]));
        let execution_positions = Arc::new(Mutex::new(vec![]));

        let starting_positions: &[(char, i16, i16)] = if j == 0 {
            &initial_positions
        } else {
            &targets[j - 1].1
        };

        simulate_v2(
            target.0,
            vec![],
            vec![starting_positions.to_vec()],
            starting_positions.to_vec(),
            Arc::clone(&executions),
            Arc::clone(&execution_positions),
            &list_of_positions,
            list_of_parallel_rules,
            &target.1,
            boundaries,
            &target.2,
            &target.3,
            &rules,
            visibility,
            opacity,
            views,
        );
        println!("{} executions found!", executions.lock().unwrap().len());

        let executions_guard = executions.lock().unwrap();
        let positions_guard = execution_positions.lock().unwrap();

        /*   // Or if you also have positions:
        let (sorted_executions, sorted_activations, sorted_positions) =
            sort_execution_records(&executions_guard, &actives, &positions_guard);
        for (active, execution) in sorted_activations.iter().zip(sorted_executions.iter()) {
            println!("Active counts per execution: {:?}", active);
            println!("   {:?}", execution);
        }

        // without filtering
        executions_result.push(sorted_executions);
        execution_positions_result.push(sorted_positions);
        activations_counts_result.push(sorted_activations);*/

        // without filtering
        executions_result.push(executions_guard.clone());
        execution_positions_result.push(positions_guard.clone());
    }

    (executions_result, execution_positions_result)
}

pub fn simulation_with_all_executions(
    initial_positions: &[Position],
    steps: usize,
    list_of_parallel_rules: &Vec<ParallelRules>,
    views: &[View],
    rules: &[Rule],
    visibility: &i16,
) -> Vec<Vec<usize>> {
    let (list_of_starting_positions, list_of_ending_positions) =
        prepare_starting_and_ending_positions(list_of_parallel_rules, views, rules);
    let executions = Arc::new(Mutex::new(vec![]));

    simulate(
        steps,
        vec![],
        initial_positions.to_vec(),
        Arc::clone(&executions),
        &list_of_starting_positions,
        &list_of_ending_positions,
        list_of_parallel_rules,
        &rules,
        views,
        visibility,
    );
    let executions_result = executions.lock().unwrap().clone();
    executions_result
}

pub fn prepare_starting_and_ending_positions(
    list_of_parallel_rules: &Vec<ParallelRules>,
    views: &[View],
    rules: &[Rule],
) -> (Vec<Vec<Position>>, Vec<Vec<Position>>) {
    let mut list_of_starting_positions = vec![];
    let mut list_of_ending_positions = vec![];

    for parallel_rules in list_of_parallel_rules {
        let (starting_positions, _) = extract_starting_positions(parallel_rules, &views, &rules);
        list_of_starting_positions.push(starting_positions);

        let (ending_positions, _) = extract_ending_positions(parallel_rules, &rules);
        list_of_ending_positions.push(ending_positions);
    }

    (list_of_starting_positions, list_of_ending_positions)
}

/*

fn _simulate_v2_iterative(
    steps: usize,
    initial_execution: Vec<usize>,
    initial_execution_position: Vec<Vec<Position>>,
    initial_position: Vec<Position>,
    execution_history: Arc<Mutex<Vec<Vec<usize>>>>,
    execution_position_history: Arc<Mutex<Vec<Vec<Vec<Position>>>>>,
    list_of_positions: &Vec<Vec<DualPosition>>,
    list_of_parallel_rules: &Vec<ParallelRules>,
    goal_position: &[Position],
    rules: &[Rule]
) {
    let stack = Arc::new(Mutex::new(vec![(
        steps,
        initial_execution,
        initial_execution_position,
        initial_position,
    )]));

    while let Some((remaining_steps, execution, execution_position, current_position)) =
        stack.lock().unwrap().pop()
    {
        if remaining_steps == 0 {
            continue;
        }

        let solutions: Vec<(usize, Vec<Position>)> = search_for_solutions_m2(
            list_of_positions,
            &adjust_positions(0, &current_position),
            &current_position,
            &Some((&-3, &3, &-2, &4)),
        );

        solutions.into_par_iter().for_each(|(exe, exe_pos)| {
            if !is_compatible_parallel_rules(&execution, exe, list_of_parallel_rules, &rules)
                || has_cycle(&exe_pos, &execution_position)
            {
                return;
            }

            let mut updated_execution = execution.clone();
            updated_execution.push(exe);

            let mut updated_execution_position = execution_position.clone();
            updated_execution_position.push(exe_pos.clone());

            if move_on_ligne_goal(&exe_pos, &goal_position.to_vec()) {
                let mut history = execution_history.lock().unwrap();
                history.push(updated_execution);
                let mut pos_history = execution_position_history.lock().unwrap();
                pos_history.push(updated_execution_position);
            } else {
                stack.lock().unwrap().push((
                    remaining_steps - 1,
                    updated_execution,
                    updated_execution_position,
                    exe_pos,
                ));
            }
        });
    }
} */

/*
// Function to create a unique folder based on time or incremental ID
fn _create_unique_output_dir(base_path: &str) -> String {
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let new_dir = format!("{}/execution_{}", base_path, timestamp);
    fs::create_dir_all(&new_dir).unwrap();
    new_dir
}
*/

/*
/// Checks if all `must_exist_positions` exist in the given execution list.
fn all_must_exist(execution_list: &[(usize, i16, i16)], must_exist_positions: &[(i16, i16)]) -> bool {
    must_exist_positions.iter().all(|&(mx, my)| {
        execution_list.iter().any(|&(_, x, y)| x == mx && y == my)
    })
}

/// Filters `execution_positions` and `execution_indexes` by removing matched executions in-place.
fn filter_executions(
    execution_positions: &mut Vec<Vec<(usize, i16, i16)>>,
    execution_indexes: &mut Vec<Vec<usize>>,
    must_exist_positions: &[<(i16, i16)>],
) {
    assert_eq!(execution_positions.len(), execution_indexes.len(), "Lengths must match");

    // Use retain to modify vectors in place instead of creating new ones
    let mut index = 0;
    execution_positions.retain(|pos_list| {
        let keep = !all_must_exist(pos_list, must_exist_positions);
        if !keep {
            execution_indexes.remove(index);
        } else {
            index += 1;
        }
        keep
    });
}

fn main() {
    let mut execution_positions = vec![
        vec![(0, 1, 2), (1, 3, 4)],
        vec![(2, 5, 6), (3, 7, 8)],
        vec![(4, 1, 2), (5, 5, 6)]
    ];

    let mut execution_indexes = vec![
        vec![0, 1],
        vec![2, 3],
        vec![4, 5]
    ];

    let must_exist_positions = vec![(1, 2), (5, 6)]; // These must exist in an item to be removed

    filter_executions(&mut execution_positions, &mut execution_indexes, &must_exist_positions);

    println!("Filtered Execution Positions: {:?}", execution_positions);
    println!("Filtered Execution Indexes: {:?}", execution_indexes);
}
 */

pub fn separate_isolated_positions(
    current_position: &[Position],
    visibility: i16,
) -> (Vec<Position>, Vec<Position>) {
    let mut isolated_positions = Vec::new();
    let mut positions = Vec::new();

    'outer: for (i, pos) in current_position.iter().enumerate() {
        for (j, other) in current_position.iter().enumerate() {
            if i != j && distance(pos, other) <= visibility {
                positions.push(pos.clone());
                continue 'outer;
            }
        }
        isolated_positions.push(pos.clone());
    }

    (positions, isolated_positions)
}

// Manhattan distance
pub fn distance(a: &Position, b: &Position) -> i16 {
    let dx = (a.1 - b.1).abs();
    let dy = (a.2 - b.2).abs();
    dx + dy
}

fn sort_executions_with_activations(
    executions: &[Vec<usize>],
    activations: &[usize],
) -> (Vec<Vec<usize>>, Vec<usize>) {
    // Ensure both vectors have the same length
    assert_eq!(
        executions.len(),
        activations.len(),
        "Executions and activations vectors must have the same length"
    );

    // 🔑 SORT executions and activations together for deterministic order
    let mut paired_data: Vec<(Vec<usize>, usize)> = executions
        .iter()
        .zip(activations.iter())
        .map(|(exec, &act)| (exec.clone(), act))
        .collect();

    // Sort by execution first, then by activation count
    paired_data.sort_by(|(exec_a, act_a), (exec_b, act_b)| {
        exec_a.cmp(exec_b).then_with(|| act_a.cmp(act_b))
    });

    // Separate back into sorted executions and activations
    let sorted_executions: Vec<Vec<usize>> =
        paired_data.iter().map(|(exec, _)| exec.clone()).collect();
    let sorted_activations: Vec<usize> = paired_data.iter().map(|(_, act)| *act).collect();

    (sorted_executions, sorted_activations)
}

fn sort_execution_records(
    executions: &[Vec<usize>],
    activations: &[usize],
    positions: &[Vec<Vec<Position>>],
) -> (Vec<Vec<usize>>, Vec<usize>, Vec<Vec<Vec<Position>>>) {
    // Ensure all vectors have the same length
    assert_eq!(executions.len(), activations.len());
    assert_eq!(executions.len(), positions.len());

    // Create execution records combining all three data types
    let mut execution_records: Vec<(Vec<usize>, usize, Vec<Vec<Position>>)> = executions
        .iter()
        .zip(activations.iter())
        .zip(positions.iter())
        .map(|((exec, &act), pos)| (exec.clone(), act, pos.clone()))
        .collect();

    // Sort by execution first, then by activation count
    execution_records.sort_by(|(exec_a, act_a, _), (exec_b, act_b, _)| {
        exec_a.cmp(exec_b).then_with(|| act_a.cmp(act_b))
    });

    // Separate back into individual sorted vectors
    let sorted_executions: Vec<Vec<usize>> = execution_records
        .iter()
        .map(|(exec, _, _)| exec.clone())
        .collect();
    let sorted_activations: Vec<usize> = execution_records.iter().map(|(_, act, _)| *act).collect();
    let sorted_positions: Vec<Vec<Vec<Position>>> = execution_records
        .iter()
        .map(|(_, _, pos)| pos.clone())
        .collect();

    (sorted_executions, sorted_activations, sorted_positions)
}
