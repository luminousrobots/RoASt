use std::collections::HashSet;

use crate::methodology::view::distribute_abstract_positions;

use super::{
    color::generate_colors_combinations,
    position::{rotate_point, Position},
};

pub type View = Vec<Position>;

pub fn generate_robot_view(mut views: Vec<View>, num_robots: usize, visibility: i16) -> Vec<View> {
    let mut remaining_robots = num_robots;
    let mut processed_views = 0;

    while remaining_robots > 0 {
        let mut new_views = Vec::new();

        for view in views.iter().skip(processed_views) {
            if let Some((_, x, y)) =
                find_last_robot_position(view).or(Some(('r', 0, visibility + 1)))
            {
                for j in (-visibility..=visibility).rev().filter(|&j| j <= y) {
                    for i in (-visibility..=visibility).filter(|&i| j < y || (j == y && i > x)) {
                        if (j.abs() + i.abs() <= visibility) && (i != 0 || j != 0) {
                            let mut new_view = view.clone();
                            new_view.push(('r', i, j));
                            new_views.push(new_view);
                        }
                    }
                }
            }
        }

        processed_views = views.len();
        views.extend(new_views);
        remaining_robots -= 1;
    }

    views
}
fn find_last_robot_position(view: &View) -> Option<Position> {
    view.iter()
        .filter(|&&(_, x, y)| !(x == 0 && y == 0))
        .max_by_key(|&&(_, x, y)| (y, -x))
        .cloned()
}

pub fn remove_identical_views(views: &mut Vec<View>) {
    let mut i = 0;
    while i < views.len() {
        let mut j = i + 1;
        while j < views.len() {
            if are_equivalent_with_rotation(&views[i], &views[j]) {
                views.remove(j);
            } else {
                j += 1;
            }
        }
        i += 1;
    }
}

pub fn distribute_robot_colors_iterative(views: &mut Vec<View>, colors: &Vec<char>) {
    let initial_views_length = views.len();

    for _ in 0..initial_views_length {
        // Remove the first view from the list
        let first_view = views.remove(0);
        // Generate all color combinations for these View
        let colors_combinations = generate_colors_combinations(first_view.len(), colors);

        // Generate new views based on the color combinations
        for colors_combination in colors_combinations {
            let mut new_view = first_view.clone(); // Clone the original view
            for (i, &color) in colors_combination.iter().enumerate() {
                new_view[i].0 = color; // Update the color
            }
            views.push(new_view); // Add the new view back to the list
        }
    }
}

pub fn rotate_view(view: &View, angle: i16) -> View {
    view.iter()
        .map(|&(c, x, y)| {
            let (new_x, new_y) = rotate_point(&x, &y, &angle);
            (c, new_x, new_y)
        })
        .collect()
}

pub fn are_equivalent_with_rotation(view_1: &View, view_2: &View) -> bool {
    let rotations = vec![0, 90, 180, 270];
    for angle in rotations {
        let rotated_view = rotate_view(&view_2, angle);
        if are_equivalent(&view_1, &rotated_view) {
            return true;
        }
    }
    false
}

pub fn are_equivalent(vec1: &View, vec2: &View) -> bool {
    if vec1.len() != vec2.len() {
        return false;
    }
    let set1: HashSet<_> = vec1.iter().collect();
    let set2: HashSet<_> = vec2.iter().collect();
    set1 == set2
}

pub fn are_equivalent_with_opacity(vec1: &View, vec2: &View) -> bool {
    if vec1.len() != vec2.len() {
        return false;
    }

    let mut unmatched = vec1.clone();

    for p2 in vec2 {
        if let Some(pos) = unmatched.iter().position(|p1| {
            if p2.0 == 'X' {
                // Match only on coordinates
                p1.1 == p2.1 && p1.2 == p2.2
            } else {
                p1 == p2
            }
        }) {
            unmatched.remove(pos); // consume one match
        } else {
            return false;
        }
    }

    unmatched.is_empty()
}

pub fn remove_symmetrical_rotations(views: &mut Vec<View>, opcity: bool, visibility: i16) {
    views.retain(|view| has_symmetrical_rotation(view, opcity, visibility));
}

pub fn has_symmetrical_rotation(view: &View, opcity: bool, visibility: i16) -> bool {
    let mut view_copy = view.clone();
    if opcity {
        distribute_abstract_positions(&mut view_copy, visibility);
    }
    let rotations = [
        view_copy.clone(), // Original view
        rotate_view(&view_copy, 90),
        rotate_view(&view_copy, 180),
        rotate_view(&view_copy, 270),
    ];

    // Compare each rotation with the rest
    for i in 0..rotations.len() - 1 {
        // Skip the last item
        for j in (i + 1)..rotations.len() {
            if are_equivalent(&rotations[i], &rotations[j]) {
                return false; // Found an identical view
            }
        }
    }

    true // No identical views found
}

pub fn compare_views(p1: &Position, p2: &Position) -> Option<bool> {
    if p1.1 == p2.1 && p1.2 == p2.2 {
        return if p1.0 == p2.0 { Some(true) } else { None };
    }
    Some(false)
}

pub fn get_view_from_positions(index: &usize, positions: &[Position], visibility: i16) -> View {
    let mut view = Vec::with_capacity(positions.len()); // Pre-allocate memory
    let reference = positions[*index]; // Get the reference position

    view.push(reference); // Add the reference position

    // Collect only positions within visibility range (excluding reference itself)
    view.extend(positions.iter().filter(|&&pos| {
        let distance = pos.1.abs() + pos.2.abs();
        distance <= visibility && distance != 0
    }));
    view
}

pub fn remove_existed_views(generated_views: &mut Vec<View>, existed_views: &Vec<View>) {
    /*
       for view in existed_views{
        let mut i=0;
         while i<generated_views.len() {
            if are_equivalent_with_rotation(&view, &generated_views[i]) {
                generated_views.remove(i);
                continue;
            } else {
                i += 1;
            }

         }

    } */
    generated_views.retain(|gen_view| {
        !existed_views
            .iter()
            .any(|ex_view| are_equivalent_with_rotation(gen_view, ex_view))
    });
}

pub fn exists_in_view(i: &i16, j: &i16, view: &Vec<Position>) -> bool {
    for (_, x, y) in view {
        if *i == *x && *j == *y {
            return true;
        }
    }
    false
}

/*******************************************************
 *                                                     *
 *                   Display Logs                      *
 *                                                     *
 *******************************************************/

pub fn display_view(view: &View, visibility: &i16) {
    for y in (-visibility..=*visibility).rev() {
        for x in -visibility..=*visibility {
            if x.abs() + y.abs() <= *visibility {
                if let Some(&(id, _, _)) = view.iter().find(|&&(_, vx, vy)| vx == x && vy == y) {
                    print!(" {:<2} ", id);
                } else {
                    print!(" .  ");
                }
            } else {
                print!("    ");
            }
        }
        println!();
    }
}
