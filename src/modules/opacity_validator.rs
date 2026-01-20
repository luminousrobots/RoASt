use crate::{
    methodology::view::apply_abstract_positions_distribution,
    modules::{
        position::Position,
        rule::Rule,
        view::{are_equivalent_with_rotation, display_view, View},
    },
};

pub fn generate_group_views_by_opacity(
    views: &[View],
    visibility: i16,
) -> (Vec<Vec<usize>>, Vec<Vec<Position>>) {
    let mut groups: Vec<Vec<usize>> = Vec::new();

    let mut views_with_opacity: Vec<Vec<Position>> = views.to_vec();
    apply_abstract_positions_distribution(&mut views_with_opacity, visibility);

    /* for (i, v) in &mut views.iter().enumerate() {
        println!("*******************{}*********************", i);
        display_view(v, &visibility);
    }
    for (i, v) in &mut views_with_opacity.iter().enumerate() {
        println!("-------------------{}--------------------", i);
        display_view(v, &visibility);
    }*/

    for (i, view) in views_with_opacity.iter().enumerate() {
        for (j, other_view) in views_with_opacity.iter().enumerate().skip(i + 1) {
            if are_equivalent_with_rotation(view, other_view) {
                if let Some(group) = groups.iter_mut().find(|g| g.contains(&i) || g.contains(&j)) {
                    if !group.contains(&i) {
                        group.push(i);
                    }
                    if !group.contains(&j) {
                        group.push(j);
                    }
                } else {
                    groups.push(vec![i, j]);
                }
            }
        }
    }

    (groups, views_with_opacity)
}
/*
//not used for now
pub fn _remove_duplicate_opacity_views(
    mut original_views: Vec<View>,
    mut generated_views: Vec<View>,
    visibility: i16
) -> Vec<View> {
    apply_abstract_positions_distribution(&mut original_views, visibility);
    apply_abstract_positions_distribution(&mut generated_views, visibility);

    if original_views.len() != original_views.len() {
        println!("Error: mismatch in original_views length");
        return generated_views;
    }

    if generated_views.len() != generated_views.len() {
        println!("Error: mismatch in generated_views length");
        return generated_views;
    }

    generated_views.retain(|g| {
        !original_views.iter().any(|o| are_equivalent_with_rotation(o, g))
    });

    generated_views
}

*/

pub fn are_belong_to_same_opacity_group(
    index1: usize,
    index2: usize,
    views_grouped_by_opacity: &[Vec<usize>],
) -> bool {
    for group in views_grouped_by_opacity {
        if group.contains(&index1) && group.contains(&index2) {
            return true;
        }
    }
    false
}

pub fn are_conflicting_by_opacity(
    rule_a: &Rule,
    rule_b: &Rule,
    views_grouped_by_opacity: &[Vec<usize>],
) -> bool {
    // rule_a.view_id != rule_b.view_id &&
    are_belong_to_same_opacity_group(rule_a.view_id, rule_b.view_id, views_grouped_by_opacity)
        && (rule_a.direction != rule_b.direction || rule_a.color != rule_b.color)
}
