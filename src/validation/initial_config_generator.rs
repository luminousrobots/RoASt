use crate::methodology::{configuration::CONFIG, parallel_rules_viewer::Position};

pub type CompactView = Vec<Position>;
pub type InitialConfig = (Vec<Position>, bool);

pub fn generate_initial_configs(
    compact_views: Vec<CompactView>,
    leader_colors: Vec<char>,
    visibility_range: i16,
    is_obstacle_opaque: bool,
) -> Vec<InitialConfig> {
    let mut all_configs = Vec::new();

    for view in compact_views {
        let configs =
            generate_configs_for_view(&view, &leader_colors, visibility_range, is_obstacle_opaque);
        all_configs.extend(configs);
    }

    all_configs
}

fn generate_configs_for_view(
    robots: &[Position],
    leader_colors: &[char],
    visibility: i16,
    robots_must_see_each_other: bool,
) -> Vec<InitialConfig> {
    let mut configs = Vec::new();

    let (min_x, max_x, min_y, max_y) = calculate_bounds(robots, visibility);

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            if robots.iter().any(|(_, rx, ry)| *rx == x && *ry == y) {
                continue;
            }

            let mut at_least_one_robot_sees_obstacle = false;
            for (_, rx, ry) in robots {
                let distance = (x - rx).abs() + (y - ry).abs();
                if distance <= visibility {
                    at_least_one_robot_sees_obstacle = true;
                    break;
                }
            }

            if !at_least_one_robot_sees_obstacle {
                continue;
            }

            let mut config = vec![(CONFIG.obstacle, x, y)];
            config.extend_from_slice(robots);

            if robots_must_see_each_other && !check_robots_visibility(&config) {
                continue;
            }

            let is_leader = check_leader_configuration(&config, visibility, leader_colors);

            let obstacle_x = x;
            let obstacle_y = y;
            let normalized_config: Vec<Position> = config
                .iter()
                .map(|(ch, px, py)| (*ch, px - obstacle_x, py - obstacle_y))
                .collect();

            configs.push((normalized_config, is_leader));
        }
    }

    configs
}

fn calculate_bounds(robots: &[Position], visibility: i16) -> (i16, i16, i16, i16) {
    let mut min_x = i16::MAX;
    let mut max_x = i16::MIN;
    let mut min_y = i16::MAX;
    let mut max_y = i16::MIN;

    for &(_, x, y) in robots {
        min_x = min_x.min(x - visibility);
        max_x = max_x.max(x + visibility);
        min_y = min_y.min(y - visibility);
        max_y = max_y.max(y + visibility);
    }

    (min_x, max_x, min_y, max_y)
}

fn check_robots_visibility(config: &[Position]) -> bool {
    let obstacle = config.iter().find(|(ch, _, _)| *ch == CONFIG.obstacle);
    if obstacle.is_none() {
        return true;
    }

    let (_, ox, oy) = *obstacle.unwrap();
    let robots: Vec<&Position> = config
        .iter()
        .filter(|(ch, _, _)| *ch != CONFIG.obstacle)
        .collect();

    if robots.len() < 2 {
        return true;
    }

    for i in 0..robots.len() {
        for j in (i + 1)..robots.len() {
            let (_, x1, y1) = *robots[i];
            let (_, x2, y2) = *robots[j];

            if is_obstacle_blocking(x1, y1, x2, y2, ox, oy) {
                return false;
            }
        }
    }

    true
}

fn is_obstacle_blocking(x1: i16, y1: i16, x2: i16, y2: i16, ox: i16, oy: i16) -> bool {
    if (ox == x1 && oy == y1) || (ox == x2 && oy == y2) {
        return false;
    }

    let same_line = x1 == x2 || y1 == y2;

    if !same_line {
        return false;
    }

    if x1 == x2 && x1 == ox {
        let min_y = y1.min(y2);
        let max_y = y1.max(y2);
        return oy > min_y && oy < max_y;
    } else if y1 == y2 && y1 == oy {
        let min_x = x1.min(x2);
        let max_x = x1.max(x2);
        return ox > min_x && ox < max_x;
    }

    false
}

fn check_leader_configuration(
    config: &[Position],
    visibility: i16,
    leader_colors: &[char],
) -> bool {
    let obstacle = config.iter().find(|(ch, _, _)| *ch == CONFIG.obstacle);
    if obstacle.is_none() {
        return false;
    }

    let (_, ox, oy) = *obstacle.unwrap();
    let robots: Vec<&Position> = config
        .iter()
        .filter(|(ch, _, _)| *ch != CONFIG.obstacle)
        .collect();

    // 1. Find who sees the obstacle
    let visible_robots: Vec<&Position> = robots
        .iter()
        .filter(|(_, rx, ry)| {
            let distance = (ox - *rx).abs() + (oy - *ry).abs();
            distance <= visibility
        })
        .copied()
        .collect();

    // CHECK 1: Existence - at least one robot must see the obstacle
    if visible_robots.is_empty() {
        return false;
    }

    // Validate EVERY visible robot against the rules
    for (robot_color, robot_x, robot_y) in visible_robots {
        // CHECK 2: Is it a Leader?
        if !leader_colors.contains(robot_color) {
            return false;
        }

        // CHECK 3: Is it exactly at the edge of vision?
        let distance = (ox - *robot_x).abs() + (oy - *robot_y).abs();
        if distance != visibility {
            return false;
        }

        // CHECK 4: Is the robot approaching (not passed)?
        // (Robot must be to the left or same X as Obstacle)
        if *robot_x > ox {
            return false;
        }
    }

    true
}
