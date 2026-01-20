use chrono::Local;
use once_cell::sync::Lazy;
use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    sync::{RwLock, RwLockReadGuard},
};

use crate::modules::{parallel_rules::ParallelRules, rule::Rule, view::View};

/// Global storage
static VIEWS: Lazy<RwLock<Vec<View>>> = Lazy::new(|| RwLock::new(vec![]));
static RULES: Lazy<RwLock<Vec<Rule>>> = Lazy::new(|| RwLock::new(vec![]));
static PARALLEL_RULES: Lazy<RwLock<Vec<ParallelRules>>> = Lazy::new(|| RwLock::new(vec![]));
static NUMBER_OF_ROBOTS: Lazy<RwLock<usize>> = Lazy::new(|| RwLock::new(0));
static NUMBER_OF_COLORS: Lazy<RwLock<usize>> = Lazy::new(|| RwLock::new(0));
static VISIBILITY_RANGE: Lazy<RwLock<i16>> = Lazy::new(|| RwLock::new(0));
static ALL_COLOR_LETTERS: Lazy<RwLock<Vec<char>>> = Lazy::new(|| RwLock::new(vec![]));

static VIEWS_GROUPED_BY_OPACITY: Lazy<RwLock<Vec<Vec<usize>>>> = Lazy::new(|| RwLock::new(vec![]));
static VIEWS_WITH_OPACITY: Lazy<RwLock<Vec<View>>> = Lazy::new(|| RwLock::new(vec![]));

// O(1) lookup for opacity groups: view_id -> group_id
static OPACITY_GROUP_LOOKUP: Lazy<RwLock<HashMap<usize, usize>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

static ORIGINAL_VIEWS_COUNT: Lazy<RwLock<usize>> = Lazy::new(|| RwLock::new(0));
static ORIGINAL_RULES_COUNT: Lazy<RwLock<usize>> = Lazy::new(|| RwLock::new(0));
static ON_SPACE_VIEWS: Lazy<RwLock<Vec<View>>> = Lazy::new(|| RwLock::new(vec![]));

static EXEC_ROOT: Lazy<RwLock<Option<PathBuf>>> = Lazy::new(|| RwLock::new(None));

/// **Set functions**
pub fn set_views(views: Vec<View>) {
    let mut global_views = VIEWS.write().unwrap();
    *global_views = views;
}

pub fn set_rules(rules: Vec<Rule>) {
    let mut global_rules = RULES.write().unwrap();
    *global_rules = rules;
}

pub fn set_parallel_rules(parallel_rules: Vec<ParallelRules>) {
    let mut global_parallel_rules = PARALLEL_RULES.write().unwrap();
    *global_parallel_rules = parallel_rules;
}

pub fn set_views_grouped_by_opacity_(views_grouped_by_opacity: Vec<Vec<usize>>) {
    // Build O(1) lookup map: view_id -> group_id
    let mut lookup = HashMap::new();
    for (group_id, view_ids) in views_grouped_by_opacity.iter().enumerate() {
        for &view_id in view_ids {
            lookup.insert(view_id, group_id);
        }
    }

    // Store both the groups and the lookup map
    let mut global_views_grouped_by_opacity = VIEWS_GROUPED_BY_OPACITY.write().unwrap();
    *global_views_grouped_by_opacity = views_grouped_by_opacity;

    let mut global_lookup = OPACITY_GROUP_LOOKUP.write().unwrap();
    *global_lookup = lookup;
}

pub fn set_original_views_count(count: usize) {
    let mut lock = ORIGINAL_VIEWS_COUNT.write().unwrap();
    *lock = count;
}

// Setter for original rules
pub fn set_original_rules_count(count: usize) {
    let mut lock = ORIGINAL_RULES_COUNT.write().unwrap();
    *lock = count;
}

// Setter for on-space views (extended views)
pub fn set_on_space_views(views: Vec<View>) {
    let mut lock = ON_SPACE_VIEWS.write().unwrap();
    *lock = views;
}

pub fn set_views_with_opacity_(views: Vec<View>) {
    let mut lock = VIEWS_WITH_OPACITY.write().unwrap();
    *lock = views;
}

/// **Add function for views**
pub fn add_views(new_views: &[View]) {
    let mut global_views = VIEWS.write().unwrap();
    global_views.extend_from_slice(new_views);
}

/// **Add function for rules**
pub fn add_rules(new_rules: &[Rule]) {
    let mut global_rules = RULES.write().unwrap();
    global_rules.extend_from_slice(new_rules);
}

/// **Add function for parallel rules**
pub fn add_parallel_rules(new_parallel_rules: &[ParallelRules]) {
    let mut global_parallel_rules = PARALLEL_RULES.write().unwrap();
    global_parallel_rules.extend_from_slice(new_parallel_rules);
}

pub fn set_number_of_robots(number_of_robots: usize) {
    let mut global_number_of_robots = NUMBER_OF_ROBOTS.write().unwrap();
    *global_number_of_robots = number_of_robots;
}

pub fn set_number_of_colors(number_of_colors: usize) {
    let mut global_number_of_colors = NUMBER_OF_COLORS.write().unwrap();
    *global_number_of_colors = number_of_colors;
}

pub fn set_visibility(visibility_range: i16) {
    let mut global_visibility_range = VISIBILITY_RANGE.write().unwrap();
    *global_visibility_range = visibility_range;
}

pub fn set_all_color_letters(all_color_letters: Vec<char>) {
    let mut global_all_color_letters = ALL_COLOR_LETTERS.write().unwrap();
    *global_all_color_letters = all_color_letters;
}

/// **Get functions (return references, not clones!)**
pub fn get_views() -> RwLockReadGuard<'static, Vec<View>> {
    VIEWS.read().unwrap()
}

pub fn get_rules() -> RwLockReadGuard<'static, Vec<Rule>> {
    RULES.read().unwrap()
}

pub fn get_parallel_rules() -> RwLockReadGuard<'static, Vec<ParallelRules>> {
    PARALLEL_RULES.read().unwrap()
}

pub fn get_number_of_robots() -> RwLockReadGuard<'static, usize> {
    NUMBER_OF_ROBOTS.read().unwrap()
}

pub fn get_number_of_colors() -> RwLockReadGuard<'static, usize> {
    NUMBER_OF_COLORS.read().unwrap()
}

pub fn get_visibility() -> RwLockReadGuard<'static, i16> {
    VISIBILITY_RANGE.read().unwrap()
}

pub fn get_all_color_letters() -> Vec<char> {
    ALL_COLOR_LETTERS.read().unwrap().clone()
}
pub fn get_views_grouped_by_opacity_() -> RwLockReadGuard<'static, Vec<Vec<usize>>> {
    VIEWS_GROUPED_BY_OPACITY.read().unwrap()
}

pub fn get_views_with_opacity_() -> RwLockReadGuard<'static, Vec<View>> {
    VIEWS_WITH_OPACITY.read().unwrap()
}

/// Get the opacity group ID (family ID) for a view using O(1) lookup
/// Returns Some(group_id) if view is part of an opacity group (rotation family)
/// Returns None if view is standalone (not part of any rotation family)
pub fn get_opacity_group_id(view_id: usize) -> Option<usize> {
    OPACITY_GROUP_LOOKUP.read().unwrap().get(&view_id).copied()
}

/// Get the opacity group lookup map for batch operations
/// This allows multiple lookups without re-acquiring the lock
pub fn get_opacity_group_lookup() -> RwLockReadGuard<'static, HashMap<usize, usize>> {
    OPACITY_GROUP_LOOKUP.read().unwrap()
}

/// Check if two views belong to the same opacity group (same rotation family)
/// Returns true if both views are rotations of each other
pub fn are_in_same_opacity_group(view_id1: usize, view_id2: usize) -> bool {
    let lookup = OPACITY_GROUP_LOOKUP.read().unwrap();
    match (lookup.get(&view_id1), lookup.get(&view_id2)) {
        (Some(group1), Some(group2)) => group1 == group2,
        _ => false,
    }
}

/// Get all views in the same opacity group as the given view
/// Returns the complete "family" of rotation views
/// Returns None if the view is not part of any opacity group
pub fn get_opacity_group_members(view_id: usize) -> Option<Vec<usize>> {
    if let Some(group_id) = get_opacity_group_id(view_id) {
        let groups = VIEWS_GROUPED_BY_OPACITY.read().unwrap();
        groups.get(group_id).cloned()
    } else {
        None
    }
}

// Getter for original views
pub fn get_original_views_count() -> usize {
    let lock = ORIGINAL_VIEWS_COUNT.read().unwrap();
    *lock
}

// Getter for original rules
pub fn get_original_rules_count() -> usize {
    let lock = ORIGINAL_RULES_COUNT.read().unwrap();
    *lock
}

// Getter for on-space views
pub fn get_on_space_views() -> Vec<View> {
    let lock = ON_SPACE_VIEWS.read().unwrap();
    lock.clone()
}

// Helper to compute total rules (original + new ones)
pub fn get_total_rules_count(new_rules_len: usize) -> usize {
    get_original_rules_count() + new_rules_len
}

pub fn init_execution_root() -> PathBuf {
    let mut guard = EXEC_ROOT.write().unwrap();

    if guard.is_none() {
        let timestamp = Local::now()
            .format("Execution_%Y-%m-%d_%H-%M-%S")
            .to_string();

        // Place inside results folder
        let folder = PathBuf::from("results").join(timestamp);
        fs::create_dir_all(&folder).expect("Failed to create execution directory");
        *guard = Some(folder);
    }

    guard.clone().unwrap()
}

pub fn get_execution_root() -> PathBuf {
    EXEC_ROOT
        .read()
        .unwrap()
        .clone()
        .expect("Execution root not initialized. Call init_execution_root() first.")
}

/// Convenience: return execution root as String for APIs expecting &str.
pub fn get_execution_root_str() -> String {
    get_execution_root().to_string_lossy().into_owned()
}
