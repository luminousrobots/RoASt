pub fn get_colors(all_color_letters: &[char], number_of_robots: usize) -> Vec<char> {
    all_color_letters
        .iter()
        .take(number_of_robots)
        .cloned()
        .collect()
}

pub fn generate_colors_combinations(number_robots: usize, colors: &Vec<char>) -> Vec<Vec<char>> {
    let mut combinations = Vec::new(); // This will store the final list of combinations
    let mut robots = vec![' '; number_robots]; // Initialize a vector to store colors of robots

    // Generate combinations recursively starting from the first robot (index 0)
    generate_colors_combinations_recursive(
        0,
        number_robots,
        &colors,
        &mut robots,
        &mut combinations,
    );

    combinations // Return the list of all combinations
}

fn generate_colors_combinations_recursive(
    index: usize,
    number_robots: usize,
    colors: &[char],
    robots: &mut Vec<char>,
    combinations: &mut Vec<Vec<char>>,
) {
    if index == number_robots {
        combinations.push(robots.clone()); // If all robots have colors, store the combination
        return; // Return, end the recursion
    }

    for &color in colors {
        // Iterate through each color
        robots[index] = color; // Assign color to the robot at `index`
        generate_colors_combinations_recursive(
            index + 1,
            number_robots,
            colors,
            robots,
            combinations,
        );
        // Move to the next robot
    }
}
