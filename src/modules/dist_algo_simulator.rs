use crate::modules::{
    direction::{calculate_movement, rotate_direction, Direction},
    final_rule::FinalRule,
    validation_config::ValidationConfig,
    view::{are_equivalent, rotate_view},
};

pub fn simulate_step(
    robots_history: &mut Vec<Vec<(char, i16, i16)>>,
    final_rules: &[FinalRule],
    config: &ValidationConfig,
    visibility: i16,
) -> bool {
    let mut queue: Vec<(char, i16, i16)> = vec![];
    let mut is_blocked = true;

    if let Some(last_state) = robots_history.last() {
        for (i, robot) in last_state.iter().enumerate() {
            // Clone all other robots except the one at index i
            let mut other_robots = last_state.clone();
            other_robots.remove(i);
            //    println!("Robot: {:?}", robot);

            let robot_view = calculate_view(*robot, &other_robots, visibility, &config);
            //display_view(&robot_view, &visibility);
            //     println!("Robot view: {:?}", robot_view);
            if let Some((dir, color)) = find_matching_rule(&robot_view, final_rules) {
                let (x, y) = calculate_movement(&dir, &robot.1, &robot.2);
                queue.push((color, x, y));
                is_blocked = false;
            } else {
                queue.push(*robot);
            }
        }
        robots_history.push(queue);
    }
    is_blocked
}

fn find_matching_rule(
    robot_view: &Vec<(char, i16, i16)>,
    final_rules: &[FinalRule],
) -> Option<(Direction, char)> {
    let rotations_angles = vec![0, 90, 180, 270];
    let suspected_final_rules = get_suspected_final_rules(&robot_view[0].0, final_rules);
    //  println!("Suspected rules len: {:?}", suspected_final_rules.len());
    /*  for (i, rule) in suspected_final_rules.iter().enumerate() {
        println!("-----------------Suspected rule {}----------------: ", i);
        print_final_rule(rule, visibility);
    }*/
    let mut matched_rule: Option<(Direction, char)> = None;
    let mut match_count = 0;

    for rule in suspected_final_rules {
        let rule_view = rule.view.clone();
        let rule_direction = rule.direction.clone();
        let rule_color = rule.color;

        for &angle in rotations_angles.iter() {
            let rotated_view = rotate_view(&rule_view, angle);
            let rotated_direction = rotate_direction(&rule_direction, angle);

            if are_equivalent(robot_view, &rotated_view) {
                match_count += 1;

                if match_count > 1 {
                    panic!("Multiple matched rules found for the given robot view!");
                }

                matched_rule = Some((rotated_direction, rule_color));
            }
        }
    }

    matched_rule
}

pub fn calculate_view(
    robot: (char, i16, i16),
    other_robots: &Vec<(char, i16, i16)>,
    visibility: i16,
    validation_config: &ValidationConfig,
) -> Vec<(char, i16, i16)> {
    let (ch, robot_x, robot_y) = robot; // Extract the character and coordinates of the robot
    let mut robots_view: Vec<(char, i16, i16)> = vec![(ch, 0, 0)];

    /*  let mut i = 0;
        while i < other_robots.len() {
            let temp_ch = other_robots[i].0;
            let temp_x = other_robots[i].1;
            let temp_y = other_robots[i].2;

            robots_view.push((temp_ch, temp_x - robot_x, temp_y - robot_y));

            i += 1;
        }
    */
    for &(ch, x, y) in other_robots {
        // Calculate the relative position
        let a = x - robot_x;
        let b = y - robot_y;

        // Check if the robot is within the visibility range
        if a.abs() + b.abs() <= visibility {
            robots_view.push((ch, a, b));
        }
    }

    // Check for positions within visibility and on the bounds
    for j in -visibility..=visibility {
        for i in -visibility..=visibility {
            if i.abs() + j.abs() <= visibility {
                let global_x = robot_x + i;
                let global_y = robot_y + j;

                // Check if the position is on the bounds
                if global_x == validation_config.min_x
                    || global_x == validation_config.max_x
                    || global_y == validation_config.min_y
                    || global_y == validation_config.max_y
                {
                    robots_view.push(('W', i, j)); // Add 'W' for positions on the bounds
                }
            }
        }
    }

    robots_view
}

fn get_suspected_final_rules(robot_marker: &char, final_rules: &[FinalRule]) -> Vec<FinalRule> {
    final_rules
        .iter()
        .enumerate()
        .filter_map(|(_, rule)| {
            if rule.view[0].0 != *robot_marker {
                return None;
            }
            Some(rule.clone()) // Return the rule itself
        })
        .collect()
}
