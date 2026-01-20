use super::position::Position;

pub fn generate_grids(
    starting_positions: &[Position],
    ending_positions: &[Position],
    visibility: &i16,
    number_of_robots: &usize,
) {
    let length = number_of_robots * *visibility as usize * 2 + 1;
    let mut starting_grid = vec![vec!['-'; length]; length];
    let mut ending_grid = vec![vec!['-'; length]; length];
    let center: usize = length as usize / 2;

    for &(ch, x, y) in starting_positions {
        let grid_x = center as isize + x as isize;
        let grid_y = center as isize - y as isize;

        if grid_x >= 0 && grid_x < length as isize && grid_y >= 0 && grid_y < length as isize {
            starting_grid[grid_y as usize][grid_x as usize] = ch;
        }
    }

    for &(ch, x, y) in ending_positions {
        let grid_x = center as isize + x as isize;
        let grid_y = center as isize - y as isize;

        if grid_x >= 0 && grid_x < length as isize && grid_y >= 0 && grid_y < length as isize {
            ending_grid[grid_y as usize][grid_x as usize] = ch;
        }
    }

    println!("Starting Grid vs Ending Grid:");
    for i in 0..length as usize {
        println!("{:?}         {:?}", starting_grid[i], ending_grid[i]);
    }
}
