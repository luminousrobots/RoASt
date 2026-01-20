use serde::{Deserialize, Serialize};

use crate::modules::{
    final_rule,
    parallel_rules::{self, extract_rules, ParallelRules},
    rule::{self, Rule},
    view::display_view,
};

use super::{direction::Direction, view::View};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FinalRule {
    pub view: View,
    pub direction: Direction,
    pub color: char,
}
pub fn print_final_rule(rule: &FinalRule, visibility: i16) {
    // println!("**********************");
    println!("View:");
    display_view(&rule.view, &visibility);

    println!("Direction:{:?}", rule.direction);
    println!("Color:{:?}", rule.color);
    println!();
}

pub fn convert_algorithms_to_final_rules(
    algorithms: &Vec<Vec<usize>>,
    list_of_parallel_rules: &Vec<ParallelRules>,
    original_rules_count: usize,
    rules: &Vec<Rule>,
    views: &Vec<View>,
) -> Vec<Vec<FinalRule>> {
    algorithms
        .iter()
        .map(|algorithm| {
            convert_algorithm_to_final_rules_v2(
                algorithm,
                list_of_parallel_rules,
                original_rules_count,
                rules,
                views,
            )
        })
        .collect()
}
/*
fn convert_algorithm_to_final_rules(
    algorithm: &Vec<usize>,
    list_of_parallel_rules: &Vec<ParallelRules>,
    rules: &Vec<rule::Rule>,
    views: &Vec<View>,
    visibility: i16,
) -> Vec<FinalRule> {
    let mut final_rules = Vec::new();
    for parallel_rules_index in algorithm {
        for rule_index in list_of_parallel_rules[*parallel_rules_index].rules.iter() {
            let rule = &rules[rule_index.0];
            final_rules.push(FinalRule {
                view: views[rule.view_id].clone(),
                direction: rule.direction,
                color: rule.color,
            });
        }
    }
    final_rules
}*/

fn convert_algorithm_to_final_rules_v2(
    algorithm: &[usize],
    list_of_parallel_rules: &[ParallelRules],
    original_rules_count: usize,
    rules: &[Rule],
    views: &[View],
) -> Vec<FinalRule> {
    let mut final_rules = Vec::new();
    let existed_rules_len = original_rules_count;
    for i in 0..existed_rules_len {
        let rule = &rules[i];
        final_rules.push(FinalRule {
            view: views[rule.view_id].clone(),
            direction: rule.direction,
            color: rule.color,
        });
    }

    let new_rules = extract_rules(algorithm, list_of_parallel_rules);
    for rule_id in new_rules {
        if rule_id > existed_rules_len - 1 {
            let rule = rules[rule_id].clone();
            final_rules.push(FinalRule {
                view: views[rule.view_id].clone(),
                direction: rule.direction,
                color: rule.color,
            });
        }
    }

    final_rules
}
