#[derive(Debug, PartialEq)]
pub enum Rotation {
    UP,
    RIGHT,
    DOWN,
    LEFT,
}

impl Rotation {
    pub fn clockwise(&self) -> Rotation {
        match self {
            Rotation::UP => Rotation::RIGHT,
            Rotation::RIGHT => Rotation::DOWN,
            Rotation::DOWN => Rotation::LEFT,
            Rotation::LEFT => Rotation::UP,
        }
    }

    pub fn anticlockwise(&self) -> Rotation {
        match self {
            Rotation::UP => Rotation::LEFT,
            Rotation::RIGHT => Rotation::UP,
            Rotation::DOWN => Rotation::RIGHT,
            Rotation::LEFT => Rotation::DOWN,
        }
    }

    pub fn to_mat(&self, d_size: (u32, u32), i_size: (u32, u32)) -> [[f32; 2]; 2] {
        match self {
            Rotation::UP => {
                if ((d_size.0 * i_size.1) as f32 / (d_size.1 * i_size.0) as f32) > 1.0 {
                    [
                        [
                            (d_size.1 * i_size.0) as f32 / (d_size.0 * i_size.1) as f32,
                            0.0,
                        ],
                        [0.0, -1.0],
                    ]
                } else {
                    [
                        [1.0, 0.0],
                        [
                            0.0,
                            -((d_size.0 * i_size.1) as f32 / (d_size.1 * i_size.0) as f32),
                        ],
                    ]
                }
            }
            Rotation::RIGHT => {
                if ((d_size.0 * i_size.0) as f32 / (d_size.1 * i_size.1) as f32) < 1.0 {
                    [
                        [
                            0.0,
                            -((d_size.0 * i_size.0) as f32 / (d_size.1 * i_size.1) as f32),
                        ],
                        [-1.0, 0.0],
                    ]
                } else {
                    [
                        [0.0, -1.0],
                        [
                            -((d_size.1 * i_size.1) as f32 / (d_size.0 * i_size.0) as f32),
                            0.0,
                        ],
                    ]
                }
            }
            Rotation::DOWN => {
                if ((d_size.0 * i_size.1) as f32 / (d_size.1 * i_size.0) as f32) > 1.0 {
                    [
                        [
                            -((d_size.1 * i_size.0) as f32 / (d_size.0 * i_size.1) as f32),
                            0.0,
                        ],
                        [0.0, 1.0],
                    ]
                } else {
                    [
                        [-1.0, 0.0],
                        [
                            0.0,
                            ((d_size.0 * i_size.1) as f32 / (d_size.1 * i_size.0) as f32),
                        ],
                    ]
                }
            }
            Rotation::LEFT => {
                if ((d_size.0 * i_size.0) as f32 / (d_size.1 * i_size.1) as f32) < 1.0 {
                    [
                        [
                            0.0,
                            ((d_size.0 * i_size.0) as f32 / (d_size.1 * i_size.1) as f32),
                        ],
                        [1.0, 0.0],
                    ]
                } else {
                    [
                        [0.0, 1.0],
                        [
                            ((d_size.1 * i_size.1) as f32 / (d_size.0 * i_size.0) as f32),
                            0.0,
                        ],
                    ]
                }
            }
        }
    }
}

#[cfg(test)]
mod main_tests {
    use super::*;

    #[test]
    fn test_clockwise_rotations() {
        assert_eq!(Rotation::UP.clockwise(), Rotation::RIGHT);
        assert_eq!(Rotation::RIGHT.clockwise(), Rotation::DOWN);
        assert_eq!(Rotation::DOWN.clockwise(), Rotation::LEFT);
        assert_eq!(Rotation::LEFT.clockwise(), Rotation::UP);
    }

    #[test]
    fn test_anticlockwise_rotations() {
        assert_eq!(Rotation::UP.anticlockwise(), Rotation::LEFT);
        assert_eq!(Rotation::RIGHT.anticlockwise(), Rotation::UP);
        assert_eq!(Rotation::DOWN.anticlockwise(), Rotation::RIGHT);
        assert_eq!(Rotation::LEFT.anticlockwise(), Rotation::DOWN);
    }
}
