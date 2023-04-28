#[derive(Debug, PartialEq)]
pub enum Rotation {
    Up,
    Right,
    Down,
    Left,
}

impl Rotation {
    pub fn clockwise(&self) -> Rotation {
        match self {
            Rotation::Up => Rotation::Right,
            Rotation::Right => Rotation::Down,
            Rotation::Down => Rotation::Left,
            Rotation::Left => Rotation::Up,
        }
    }

    pub fn anticlockwise(&self) -> Rotation {
        match self {
            Rotation::Up => Rotation::Left,
            Rotation::Right => Rotation::Up,
            Rotation::Down => Rotation::Right,
            Rotation::Left => Rotation::Down,
        }
    }

    pub fn to_mat(&self, d_size: (u32, u32), i_size: (u32, u32)) -> [[f32; 2]; 2] {
        let (mut t, mut s) = match self {
            Rotation::Up | Rotation::Down => (
                (d_size.1 * i_size.0) as f32 / (d_size.0 * i_size.1) as f32,
                (d_size.0 * i_size.1) as f32 / (d_size.1 * i_size.0) as f32,
            ),
            Rotation::Right | Rotation::Left => (
                (d_size.0 * i_size.0) as f32 / (d_size.1 * i_size.1) as f32,
                (d_size.1 * i_size.1) as f32 / (d_size.0 * i_size.0) as f32,
            ),
        };
        if t < 1.0 {
            s = 1.0;
        } else {
            t = 1.0;
        }
        match self {
            Rotation::Up => [[t, 0.0], [0.0, -s]],
            Rotation::Right => [[0.0, -t], [-s, 0.0]],
            Rotation::Down => [[-t, 0.0], [0.0, s]],
            Rotation::Left => [[0.0, t], [s, 0.0]],
        }
    }
}

#[cfg(test)]
mod rotation_tests {
    use super::*;

    #[test]
    fn test_clockwise_rotations() {
        assert_eq!(Rotation::Up.clockwise(), Rotation::Right);
        assert_eq!(Rotation::Right.clockwise(), Rotation::Down);
        assert_eq!(Rotation::Down.clockwise(), Rotation::Left);
        assert_eq!(Rotation::Left.clockwise(), Rotation::Up);
    }

    #[test]
    fn test_anticlockwise_rotations() {
        assert_eq!(Rotation::Up.anticlockwise(), Rotation::Left);
        assert_eq!(Rotation::Right.anticlockwise(), Rotation::Up);
        assert_eq!(Rotation::Down.anticlockwise(), Rotation::Right);
        assert_eq!(Rotation::Left.anticlockwise(), Rotation::Down);
    }
}
