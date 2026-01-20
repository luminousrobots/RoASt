use core::time;
use std::{
    fs::exists,
    process::exit,
    sync::{Arc, Mutex},
    time::Instant,
};

use rayon::vec;

use crate::{
    methodology::{
        cache::save_all,
        configuration::CONFIG,
        globals::{
            add_rules, add_views, get_all_color_letters, get_execution_root_str,
            get_number_of_colors, get_on_space_views, get_original_rules_count,
            get_original_views_count, get_parallel_rules, get_rules, get_views, get_visibility,
            set_on_space_views, set_original_rules_count, set_original_views_count,
            set_parallel_rules, set_rules, set_views,
        },
        parallel_rules,
        parallel_rules_viewer::{
            generate_parallel_rules_html, parallel_rules_to_parallel_rules_infos, Position,
        },
        rules_viewer::{create_rules_collection, generate_rules_html},
        simulator::run_simulation,
        view::{
            apply_abstract_positions_distribution, distribute_abstract_positions, generate_views,
        },
    },
    modules::{
        algorithm_manager::AlgorithmManager,
        color::get_colors,
        config::Config,
        direction::{self, rotate_direction},
        draft_rules::{self, DraftRule},
        execution_logger::log_note,
        full_rule::print_full_rule,
        opacity_validator::generate_group_views_by_opacity,
        parallel_rules::{
            extract_starting_positions, has_any_idle_robot_collision, has_rule_collision,
            print_parallel_rules, ParallelRules,
        },
        rule::{
            self, generate_rules, illuminate_new_generated_equivalent_rules_by_opacity, print_rule,
            Rule,
        },
        simulator::{simulation, simulation_with_all_executions},
        time_helper::format_elapsed_time,
        view::{
            are_equivalent, are_equivalent_with_rotation, display_view, remove_existed_views,
            rotate_view, View,
        },
    },
};

use super::{
    globals::{
        get_number_of_robots, set_all_color_letters, set_number_of_colors, set_number_of_robots,
        set_visibility,
    },
    parallel_rules::parallel_rules_generator,
};

pub fn methodology() -> usize {
    set_number_of_robots(CONFIG.number_of_robots);
    set_number_of_colors(CONFIG.number_of_colors);
    set_visibility(CONFIG.visibility_range);
    set_all_color_letters(CONFIG.all_color_letters.to_vec());
    // Assuming AlgorithmManager::new() returns a Result<AlgorithmManager, Box<dyn Error>>
    let manager = AlgorithmManager::new(CONFIG.existing_algorithm_path.as_str())
        .expect("Failed to create AlgorithmManager");
    let index: usize = 0;

    if let Some((views, rules)) = manager.extract_views_and_rules(index) {
        set_original_views_count(views.len());
        set_original_rules_count(rules.len());
        set_views(views);
        set_rules(rules);
        let on_space_rules = find_on_space_rules(&get_views());
        set_on_space_views(on_space_rules);

        let time_start = Instant::now();
        let colors: Vec<char> = get_colors(&get_all_color_letters(), *get_number_of_colors());
        let mut generated_views = generate_views(&colors);
        println!(
            "before remove existed Views len = {} ",
            generated_views.len()
        );
        log_note(&format!(
            "{} views generated in {}",
            generated_views.len(),
            format_elapsed_time(time_start)
        ));
        let time_start = Instant::now();
        remove_existed_views(&mut generated_views, &get_on_space_views());
        println!(
            "after remove existed Views len = {} ",
            generated_views.len()
        );
        log_note(&format!(
            "{} views generated after removing existing records in {}",
            generated_views.len(),
            format_elapsed_time(time_start)
        ));

        add_views(&generated_views);


        let time_start = Instant::now();
        let mut generated_rules = generate_rules(&colors, &get_views(), get_original_views_count());
        if CONFIG.opacity {

            remove_incompatible_rules_via_opacity(&get_rules(), &mut generated_rules);
        }
        add_rules(&generated_rules);
        log_note(&format!(
            "{} rules successfully generated in {}",
            generated_rules.len(),
            format_elapsed_time(time_start)
        ));

        /*/ for (i, rule) in get_rules().iter().enumerate() {
            println!("----------------------Rule {i}--------------------------");
            print_rule(rule, &get_views(), &get_visibility());
        }*/
        //  println!("views len{:?}", generate_colors_combinations(3, 2));

        println!("views len= {}", get_views().len());
        println!("rules len= {}", get_rules().len());

        log_note(&format!(
            "Views: {} generated + {} existing = {}, Rules: {} generated + {} existing = {}",
            get_views().len() - get_original_views_count(),
            get_original_views_count(),
            get_views().len(),
            get_rules().len() - get_original_rules_count(),
            get_original_rules_count(),
            get_rules().len()
        ));
    } else {
        println!("No algorithm found at index {}", index);
    }

    let time_start = Instant::now();
    let list_of_parallel_rules: Vec<ParallelRules> = parallel_rules_generator();
    set_parallel_rules(list_of_parallel_rules.clone());
    /*  print_parallel_rules(
        &list_of_parallel_rules,
        &get_visibility(),
        &get_number_of_robots(),
        &get_views(),
        &get_rules(),
    );*/


    log_note(&format!(
        "{} parallel rules successfully generated in {}",
        list_of_parallel_rules.len(),
        format_elapsed_time(time_start)
    ));

    println!("Validating parallel rules...");
    let validation_start = Instant::now();
    if !validate_parallel_rules(&list_of_parallel_rules) {
        println!("Error: Invalid parallel rules detected.");
        exit(1);
    }
    println!("All parallel rules are valid.");
    log_note(&format!(
        "Parallel rules validated in {}",
        format_elapsed_time(validation_start)
    ));

    if CONFIG.opacity {
        let (compressed_rules, removed_rules, compressed_views, original_rules_count) =
            compress_rules_by_opacity(&get_rules());
        set_rules(compressed_rules);
        set_views(compressed_views);
        set_original_rules_count(original_rules_count);

        let corrected_parallel_rules =
            correct_parallel_rules(&list_of_parallel_rules, &removed_rules);
        set_parallel_rules(corrected_parallel_rules.clone());

        log_note(&format!(
            "After compressing by opacity: Views = {}, Rules = {}, Parallel Rules = {}",
            get_views().len(),
            get_rules().len(),
            get_parallel_rules().len()
        ));
        println!("after compression by opacity:");
        println!("views len= {}", get_views().len());
        println!("rules len= {}", get_rules().len());
    }


    let collection = create_rules_collection();
    format!("{}/rules_viewer.html", get_execution_root_str());
    generate_rules_html(
        &collection,
        format!("{}/rules_viewer.html", get_execution_root_str()).as_str(),
    );

    let parallel_rules_collection = parallel_rules_to_parallel_rules_infos(&get_parallel_rules());


    generate_parallel_rules_html(
        &parallel_rules_collection,
        format!("{}/parallel_rules_viewer.html", get_execution_root_str()).as_str(),
        *get_number_of_robots(),
    );

    save_all();
    run_simulation()
}



fn find_on_space_rules(views: &[View]) -> Vec<View> {
    let mut existing_views_on_space: Vec<View> = Vec::new();

    for view in views {
        let has_wall = view.iter().any(|&(c, _, _)| c == 'W'); // Ensure 'view' has the correct structure
        if !has_wall {
            existing_views_on_space.push(view.clone());
        }
    }

    existing_views_on_space
}

fn remove_incompatible_rules_via_opacity(existing_rules: &[Rule], generated_rules: &mut Vec<Rule>) {
    for (i, e_rule) in existing_rules.iter().enumerate() {
        let mut e_rule_view = get_views()[e_rule.view_id].clone();
        distribute_abstract_positions(&mut e_rule_view, *get_visibility());

        let mut j = 0;
        while j < generated_rules.len() {
            let mut g_rule_view = get_views()[generated_rules[j].view_id].clone();
            distribute_abstract_positions(&mut g_rule_view, *get_visibility());

            let mut should_remove = false;
            let rotations = vec![0, 90, 180, 270];
            for angle in rotations {
                let rotated_view = rotate_view(&g_rule_view, angle);
                let direction = rotate_direction(&generated_rules[j].direction, angle);
                if are_equivalent(&rotated_view, &e_rule_view)
                    && (e_rule.direction != direction || e_rule.color != generated_rules[j].color)
                {
                    should_remove = true;
                    break; // Stop checking other rotations
                }
            }

            if should_remove {
                generated_rules.remove(j);
                // Don't increment j - next element shifts down to position j
            } else {
                j += 1; // Only increment if we didn't remove
            }
        }
    }
}

fn validate_parallel_rules(parallel_rules: &Vec<ParallelRules>) -> bool {
    for pr in parallel_rules {
        if !parallel_rules_validator(&pr) {
            return false;
        }
    }
    true
}
fn parallel_rules_validator(parallel_rules: &ParallelRules) -> bool {
    let (starting_positions, _) =
        extract_starting_positions(parallel_rules, &get_views(), &get_rules());
    for (i, draft_rule) in parallel_rules.rules.iter().enumerate() {
        let rule_view = &get_views()[get_rules()[draft_rule.0].view_id];
        let x = draft_rule.1;
        let y = draft_rule.2;
        let new_x = draft_rule.3;
        let new_y = draft_rule.4;

        //two loops to calculate the visibility area from (x,y)
        let mut calculated_view: Vec<Position> = Vec::new();
        for dx in -*get_visibility()..=*get_visibility() {
            for dy in -*get_visibility()..=*get_visibility() {
                if dx.abs() + dy.abs() <= *get_visibility() {
                    let __x = x + dx;
                    let __y = y + dy;
                    if let Some(&(c, vx, vy)) = starting_positions
                        .iter()
                        .find(|&&(_, vx, vy)| vx == __x && vy == __y)
                    {
                        calculated_view.push((c, dx, dy));
                    }
                }
            }
        }
        if !are_equivalent_with_rotation(rule_view, &calculated_view) {
            println!("-------------------Invalid Parallel Rule Detected--------------------");
            println!("Expected View:");
            display_view(rule_view, &get_visibility());
            println!("Calculated View:");
            display_view(&calculated_view, &get_visibility());
            println!();
            return false;
        }

        // Get all other draft rules (excluding the current one)
        let other_draft_rules: Vec<DraftRule> = parallel_rules
            .rules
            .iter()
            .enumerate()
            .filter(|(j, _)| *j != i)
            .map(|(_, dr)| dr.clone())
            .collect();
        if has_rule_collision(&other_draft_rules, &x, &y, &new_x, &new_y) {
            println!("-------------------Rule Collision Detected--------------------");
            return false;
        }
    }
    if has_any_idle_robot_collision(parallel_rules) {
        println!("-------------------Idle Robot Collision Detected--------------------");
        return false;
    }

    true
}

fn compress_rules_by_opacity(
    rules: &Vec<Rule>,
) -> (Vec<Rule>, Vec<(usize, usize)>, Vec<View>, usize) {
    let mut compressed_rules: Vec<(Rule, usize)> = rules
        .iter()
        .enumerate()
        .map(|(idx, r)| (r.clone(), idx))
        .collect();
    let mut removed_rules: Vec<(usize, usize)> = Vec::new();
    let mut original_rules_count = get_original_rules_count();

    // Phase 1: Remove duplicate rules (same view + same direction + same color)
    let mut i = 0;
    while i < compressed_rules.len() {
        let mut rule_a_view = get_views()[compressed_rules[i].0.view_id].clone();
        distribute_abstract_positions(&mut rule_a_view, *get_visibility());

        let mut j = i + 1;
        while j < compressed_rules.len() {
            let mut rule_b_view = get_views()[compressed_rules[j].0.view_id].clone();
            distribute_abstract_positions(&mut rule_b_view, *get_visibility());

            let mut is_duplicate = false;
            let rotations = vec![0, 90, 180, 270];
            for angle in rotations {
                let rotated_view = rotate_view(&rule_b_view, angle);
                let direction = rotate_direction(&compressed_rules[j].0.direction, angle);

                // Only remove if COMPLETELY identical (same view + same behavior)
                if are_equivalent(&rotated_view, &rule_a_view)
                    && compressed_rules[i].0.direction == direction
                    && compressed_rules[i].0.color == compressed_rules[j].0.color
                {
                    is_duplicate = true;
                    break;
                }
            }

            if is_duplicate {
                removed_rules.push((compressed_rules[j].1, compressed_rules[i].1));
                compressed_rules.remove(j);
                if compressed_rules[i].1 < original_rules_count {
                    original_rules_count -= 1;
                }
            } else {
                j += 1;
            }
        }
        i += 1;
    }

    // Phase 2: Deduplicate views and update all rules to use the same view_id
    let mut distributed_views: Vec<View> = Vec::new();

    for i in 0..compressed_rules.len() {
        let mut rule_view = get_views()[compressed_rules[i].0.view_id].clone();
        distribute_abstract_positions(&mut rule_view, *get_visibility());

        // Check if this view already exists in distributed_views
        let existing_view_id = distributed_views
            .iter()
            .position(|existing_view| are_equivalent_with_rotation(existing_view, &rule_view));

        let view_id = if let Some(id) = existing_view_id {
            // View already exists, use existing id
            id
        } else {
            // View doesn't exist, add it and use new id
            distributed_views.push(rule_view.clone());
            distributed_views.len() - 1
        };

        // Update the view_id to point to the distributed_views index
        compressed_rules[i].0.view_id = view_id;
    }

    // Extract just the rules without indices
    let final_rules: Vec<Rule> = compressed_rules.into_iter().map(|(r, _)| r).collect();

    (
        final_rules,
        removed_rules,
        distributed_views,
        original_rules_count,
    )
}

pub fn correct_parallel_rules(
    parallel_rules: &Vec<ParallelRules>,
    removed_rules: &Vec<(usize, usize)>,
) -> Vec<ParallelRules> {
    // Map 1: removed -> kept
    let removal_map: std::collections::HashMap<usize, usize> =
        removed_rules.iter().map(|(r, k)| (*r, *k)).collect();

    // Map 2: old_index -> new_position
    let removed_set: std::collections::HashSet<usize> =
        removed_rules.iter().map(|(r, _)| *r).collect();

    let max_id = parallel_rules
        .iter()
        .flat_map(|pr| pr.rules.iter())
        .map(|dr| dr.0)
        .max()
        .unwrap_or(0);

    let mut position_map = std::collections::HashMap::new();
    let mut pos = 0;
    for i in 0..=max_id {
        if !removed_set.contains(&i) {
            position_map.insert(i, pos);
            pos += 1;
        }
    }

    parallel_rules
        .iter()
        .map(|pr| {
            let rules = pr
                .rules
                .iter()
                .map(|dr| {
                    let mut id = dr.0;
                    // Follow removal chain
                    while let Some(&new_id) = removal_map.get(&id) {
                        id = new_id;
                    }
                    // Map to new position
                    let final_id = position_map[&id];
                    (final_id, dr.1, dr.2, dr.3, dr.4)
                })
                .collect();

            ParallelRules {
                rules,
                active_color_count: pr.active_color_count,
                movable_idle_robots: pr.movable_idle_robots.clone(),
                idle_robots_views: pr.idle_robots_views.clone(),
                active_movement_count: pr.active_movement_count,
                fixed_idle_robots: pr.fixed_idle_robots.clone(),
            }
        })
        .collect()
}
