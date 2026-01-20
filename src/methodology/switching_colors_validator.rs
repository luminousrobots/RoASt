use itertools::Itertools;

use crate::{
    methodology::{
        configuration::CONFIG,
        globals::{get_rules, get_views},
    },
    modules::{
        color::get_colors,
        direction::{self, rotate_direction},
        progress_helper::ProgressHelper,
        rule,
        view::{self, are_equivalent, rotate_view, View},
    },
};

use rayon::prelude::*;

pub fn remove_duplicates_by_color_switches(
    mut algos: Vec<Vec<usize>>,
    original_algo: &[usize],
) -> Vec<Vec<usize>> {
    let colors: Vec<char> = get_colors(&CONFIG.all_color_letters.to_vec(), CONFIG.number_of_colors);
    let possible_permutations = generate_color_permutations(&colors);
    println!(
        "Generated {} color permutations for {} colors",
        possible_permutations.len(),
        colors.len()
    );

    let progress = ProgressHelper::new(
        algos.len() as u64,
        "Cleaning duplicate algorithms by color switches ",
    );

    let mut i = 0;
    while i < algos.len() {
        let algo_a = algos[i].clone();
        let full_a: Vec<usize> = original_algo.iter().chain(algo_a.iter()).cloned().collect();

        algos = algos
            .into_par_iter()
            .enumerate()
            .filter_map(|(j, algo_b)| {
                if j <= i {
                    return Some(algo_b);
                }

                let full_b: Vec<usize> =
                    original_algo.iter().chain(algo_b.iter()).cloned().collect();

                // Parallel check for duplicates across permutations
                let is_duplicate = possible_permutations.par_iter().any(|perm| {
                    are_algorithms_equivalent_with_color_permutation(
                        &full_a,
                        &full_b,
                        perm,
                        &colors,
                        &original_algo,
                    )
                });

                if is_duplicate {
                    None
                } else {
                    Some(algo_b)
                }
            })
            .collect();

        progress.inc();
        i += 1;
    }

    progress.finish_success(
        "Cleaning duplicate algorithms by color switches",
        algos.len(),
    );
    algos
}

pub fn remove_duplicates_by_color_switches_(
    mut algos: Vec<Vec<usize>>,
    original_algo: &[usize],
) -> Vec<Vec<usize>> {
    let colors: Vec<char> = get_colors(&CONFIG.all_color_letters.to_vec(), CONFIG.number_of_colors);
    let possible_permutations = generate_color_permutations(&colors);
    println!(
        "Generated {} color permutations for {} colors",
        possible_permutations.len(),
        colors.len()
    );
    let progress = ProgressHelper::new(
        algos.len() as u64,
        "Cleaning duplicate algorithms by color switches ",
    );

    let mut i = 0;
    while i < algos.len() {
        let algo_a = algos[i].clone();
        let full_a: Vec<usize> = original_algo.iter().chain(algo_a.iter()).cloned().collect();

        let mut j = i + 1;
        while j < algos.len() {
            let algo_b = &algos[j];
            let full_b: Vec<usize> = original_algo.iter().chain(algo_b.iter()).cloned().collect();

            let mut is_duplicate = false;
            for perm in &possible_permutations {
                if are_algorithms_equivalent_with_color_permutation(
                    &full_a,
                    &full_b,
                    perm,
                    &colors,
                    &original_algo,
                ) {
                    is_duplicate = true;
                    break;
                }
            }

            if is_duplicate {
                // Remove later duplicate directly
                algos.remove(j);
            } else {
                j += 1;
            }
        }

        progress.inc();
        i += 1;
    }

    progress.finish_success(
        "Cleaning duplicate algorithms by color switches",
        algos.len(),
    );
    algos
}

/*pub fn remove_duplicates_by_color_switches(
    algos: Vec<Vec<usize>>,
    original_algo: Vec<usize>,
) -> Vec<Vec<usize>> {
    println!();
    let progress = ProgressHelper::new(
        algos.len() as u64,
        "Cleaning duplicate algorithms by color switches ",
    );
    let colors: Vec<char> = get_colors(&ALL_COLOR_LETTERS.to_vec(), NUMBER_OF_COLORS);
    let possible_permutations = generate_color_permutations(&colors);
    let mut unique_algos: Vec<Vec<usize>> = Vec::new();

    'outer: for (i, algo_a) in algos.iter().enumerate() {
        for algo_b in algos.iter().skip(i + 1) {
            // Combine with original algorithm
            let full_a: Vec<usize> = original_algo.iter().chain(algo_a.iter()).cloned().collect();
            let full_b: Vec<usize> = original_algo.iter().chain(algo_b.iter()).cloned().collect();

            // Try all color permutations
            for perm in &possible_permutations {
                if are_algorithms_equivalent_with_color_permutation(
                    &full_a,
                    &full_b,
                    perm,
                    &colors,
                    &original_algo,
                ) {
                    // Found an equivalent algorithm — skip pushing this one
                    continue 'outer;
                }
            }
        }

        // If we didn’t find any match, keep this one
        unique_algos.push(algo_a.clone());
        progress.inc();
    }

    progress.finish_success("Cleaning duplicate algorithms by color switches", unique_algos.len());
    unique_algos
}*/

pub fn are_algorithms_equivalent_with_color_permutation_shorter(
    algo_a: &[usize],
    algo_b: &[usize],
    color_permutation: &[char],
    colors: &[char],
) -> bool {
    if algo_a.len() != algo_b.len() {
        return false;
    }

    algo_a.iter().all(|&a| {
        let rule_a = &get_rules()[a];
        let view_a = &get_views()[rule_a.view_id];
        let new_view_a = apply_color_permutation_to_view(view_a.clone(), color_permutation, colors);
        let new_color_a = apply_color_permutation_to_color(rule_a.color, color_permutation, colors);

        algo_b.iter().any(|&b| {
            let rule_b = &get_rules()[b];
            let view_b = &get_views()[rule_b.view_id];
            [0, 90, 180, 270].iter().any(|&angle| {
                are_equivalent(&rotate_view(&new_view_a, angle), view_b)
                    && rotate_direction(&rule_a.direction, angle) == rule_b.direction
                    && new_color_a == rule_b.color
            })
        })
    })
}

pub fn are_algorithms_equivalent_with_color_permutation(
    algo_a: &[usize],
    algo_b: &[usize],
    color_permutation: &[char],
    colors: &[char],
    original_algo: &[usize],
) -> bool {
    // Quick check: algorithms must have the same number of rules
    if algo_a.len() != algo_b.len() {
        return false;
    }

    for (i, a) in algo_a.iter().enumerate() {
        let rule_a = &get_rules()[*a];
        let view_a = &get_views()[rule_a.view_id];
        let direction_a = rule_a.direction;
        let color_a = rule_a.color;

        // Apply color permutation to rule A
        let new_view_a = apply_color_permutation_to_view(view_a.clone(), color_permutation, colors);
        let new_color_a = apply_color_permutation_to_color(color_a, color_permutation, colors);

        // Try to find a matching rule in algo_b
        let mut found_match = false;

        for b in algo_b {
            let rule_b = &get_rules()[*b];
            let view_b = &get_views()[rule_b.view_id];
            let direction_b = rule_b.direction;
            let color_b = rule_b.color;

            // Check all 4 possible rotations (0°, 90°, 180°, 270°)
            let rotations: [i16; 4] = [0, 90, 180, 270];
            for &angle in &rotations {
                let rotated_new_view_a = rotate_view(&new_view_a, angle);
                let rotated_direction_a = rotate_direction(&direction_a, angle);

                if are_equivalent(&rotated_new_view_a, view_b)
                    && rotated_direction_a == direction_b
                    && new_color_a == color_b
                {
                    found_match = true;
                    break;
                }
            }

            if found_match {
                break;
            }
        }

        // If no matching rule found for a rule in algo_a, they differ
        if !found_match {
            return false;
        }
    }

    true
}

pub fn generate_color_permutations(colors: &[char]) -> Vec<Vec<char>> {
    let original_perm: Vec<&char> = colors.iter().collect();
    let perms = colors
        .iter()
        .permutations(colors.len())
        .filter(|p| *p != original_perm)
        .map(|p| p.into_iter().cloned().collect())
        .collect();

    perms
}
pub fn apply_color_permutation_to_view(
    view: View,
    color_permutation: &[char],
    colors: &[char],
) -> View {
    view.iter()
        .map(|&(c, x, y)| {
            (
                apply_color_permutation_to_color(c, color_permutation, colors),
                x,
                y,
            )
        })
        .collect()
}

pub fn apply_color_permutation_to_color(
    color: char,
    color_permutation: &[char],
    colors: &[char],
) -> char {
    colors
        .iter()
        .position(|&col| col == color)
        .map(|i| color_permutation[i])
        .unwrap_or(color)
}
