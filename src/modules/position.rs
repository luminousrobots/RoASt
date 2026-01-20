pub type Position = (char, i16, i16);

pub fn rotate_point(x: &i16, y: &i16, angle: &i16) -> (i16, i16) {
    match angle {
        0 => (*x, *y),
        90 => (*y, -x),
        180 => (-x, -y),
        270 => (-y, *x),
        _ => panic!("Angle must be 0, 90, 180, or 270"),
    }
}
