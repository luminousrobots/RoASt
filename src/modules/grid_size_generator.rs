fn generate_obstacles(rows: i16, cols: i16, base: i16) -> Vec<(i16, i16)> {
    let block_h = (rows - base + 1).max(1);
    let block_w = (cols - base + 1).max(1);

    let start_r = 1 + base / 2;
    let start_c = 1 + base / 2;

    let mut out = Vec::new();
    for dr in 0..block_h {
        for dc in 0..block_w {
            out.push((start_r + dr, start_c + dc));
        }
    }
    out
}

pub fn generate_grid_definitions(base: i16) -> Vec<(i16, i16, Vec<(i16, i16)>)> {
    let sizes = [
        (base, base),
        (base, base + 1),
        (base, base + 2),
        (base + 1, base),
        (base + 1, base + 1),
        (base + 1, base + 2),
        (base + 2, base),
        (base + 2, base + 1),
    ];

    let mut out = Vec::new();

    for (r, c) in sizes {
        let obs = generate_obstacles(r, c, base);
        out.push((r, c, obs));
    }

    out
}
