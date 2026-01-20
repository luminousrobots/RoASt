use crate::{
    methodology::configuration::CONFIG,
    modules::view::{
        distribute_robot_colors_iterative, exists_in_view, generate_robot_view,
        remove_identical_views, remove_symmetrical_rotations, View,
    },
};
use std::process::exit;

use super::globals::{get_number_of_robots, get_visibility};

pub fn generate_views(colors: &Vec<char>) -> Vec<View> {
    let mut views = generate_robot_view(
        vec![vec![('r', 0, 0)]],
        *get_number_of_robots() - 1,
        *get_visibility(),
    );
    remove_identical_views(&mut views);
    distribute_robot_colors_iterative(&mut views, &colors);
    views = distribute_obstacles(&views);
    //  apply_abstract_positions_distribution(&mut views);
    remove_symmetrical_rotations(&mut views, CONFIG.opacity, CONFIG.visibility_range);
    remove_identical_views(&mut views);
    views
}

pub fn distribute_obstacles(views: &Vec<View>) -> Vec<View> {
    let mut views_with_obstacles: Vec<View> = Vec::new();

    for view in views {
        views_with_obstacles.push(view.clone());
        for j in (-*get_visibility()..=*get_visibility()).rev() {
            for i in -*get_visibility()..=*get_visibility() {
                if j.abs() + i.abs() <= *get_visibility() {
                    // Only add an obstacle if (i, j) is NOT already in view
                    if !exists_in_view(&i, &j, view) {
                        let mut view_copy = view.clone();
                        view_copy.push((CONFIG.obstacle, i, j));
                        views_with_obstacles.push(view_copy);
                    }
                }
            }
        }
    }

    views_with_obstacles
}

pub fn apply_abstract_positions_distribution(views: &mut Vec<View>, visibility: i16) {
    for (i, view) in views.iter_mut().enumerate() {
        if distribute_abstract_positions(view, visibility) {}
    }
}

pub fn distribute_abstract_positions(view: &mut View, visibility: i16) -> bool {
    let mut is_distribution_modified: bool = false;

    // Positive Y axis (0, +y)
    for k in 1..=visibility {
        let obs_x = 0;
        let obs_y = k;
        let c = get_char_at(view, obs_x, obs_y);

        if c == '.' || (CONFIG.is_obstacle_opaque == false && c == CONFIG.obstacle) {
            continue;
        }

        for l in (k + 1)..=visibility {
            let behind_x = 0;
            let behind_y = l;
            set_abstract_position_at(view, behind_x, behind_y);
            is_distribution_modified = true;
        }
        break;
    }

    // Negative Y axis (0, -y)
    for k in 1..=visibility {
        let obs_x = 0;
        let obs_y = -k;
        let c = get_char_at(view, obs_x, obs_y);

        if c == '.' || (CONFIG.is_obstacle_opaque == false && c == CONFIG.obstacle) {
            continue;
        }
        for l in (k + 1)..=visibility {
            let behind_x = 0;
            let behind_y = -l;
            set_abstract_position_at(view, behind_x, behind_y);
            is_distribution_modified = true;
        }
        break;
    }

    // Positive X axis (+x, 0)
    for k in 1..=visibility {
        let obs_x = k;
        let obs_y = 0;
        let c = get_char_at(view, obs_x, obs_y);

        if c == '.' || (CONFIG.is_obstacle_opaque == false && c == CONFIG.obstacle) {
            continue;
        }
        for l in (k + 1)..=visibility {
            let behind_x = l;
            let behind_y = 0;
            set_abstract_position_at(view, behind_x, behind_y);
            is_distribution_modified = true;
        }
        break;
    }

    // Negative X axis (-x, 0)
    for k in 1..=visibility {
        let obs_x = -k;
        let obs_y = 0;

        let c = get_char_at(view, obs_x, obs_y);

        if c == '.' || (CONFIG.is_obstacle_opaque == false && c == CONFIG.obstacle) {
            continue;
        }
        for l in (k + 1)..=visibility {
            let behind_x = -l;
            let behind_y = 0;
            set_abstract_position_at(view, behind_x, behind_y);
            is_distribution_modified = true;
        }
        break;
    }
    /*   if i == 40 {
        println!("View: {:?}", view);
        display_view(view, &visibility);
    }*/
    is_distribution_modified
}

fn get_char_at(view: &View, x: i16, y: i16) -> char {
    view.iter()
        .find(|p| p.1 == x && p.2 == y) // Find a position with matching x and y
        .map_or('.', |p| p.0) // If found, return its character, otherwise return '.'
}

fn set_abstract_position_at(view: &mut View, x: i16, y: i16) -> bool {
    if let Some(pos_index) = view.iter().position(|&(_, vx, vy)| vx == x && vy == y) {
        // Position exists, check if it needs update
        if view[pos_index].0 != 'X' {
            view[pos_index].0 = 'X';
            return true; // Character updated
        }
        return false; // Already the occlusion character, no modification
    } else {
        // Position does not exist, add it
        view.push(('X', x, y));
        return true; // New occlusion added
    }
}
