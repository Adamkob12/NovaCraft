use crate::prelude::{Face, Face::*};

#[derive(Clone, Copy, Debug)]
pub enum Direction {
    North,
    South,
    East,
    West,
    NoEast,
    NoWest,
    SoEast,
    SoWest,
}

impl From<Face> for Direction {
    fn from(value: Face) -> Self {
        match value {
            Back => Self::North,
            Forward => Self::South,
            Right => Self::East,
            Left => Self::West,
            Top => {
                debug_assert!(false, "Face::Top cannot be cast as Direction");
                Self::North
            }
            Bottom => {
                debug_assert!(false, "Face::Bottom cannot be cast as Direction");
                Self::South
            }
        }
    }
}

impl Into<usize> for Direction {
    fn into(self) -> usize {
        match self {
            Direction::North => 0,
            Direction::South => 1,
            Direction::East => 2,
            Direction::West => 3,
            Direction::NoEast => 4,
            Direction::NoWest => 5,
            Direction::SoEast => 6,
            Direction::SoWest => 7,
        }
    }
}

impl Into<Face> for Direction {
    fn into(self) -> Face {
        match self {
            Self::NoWest | Self::NoEast | Direction::North => Back,
            Self::SoWest | Self::SoEast | Direction::South => Forward,
            Self::West => Left,
            Self::East => Right,
        }
    }
}

impl From<u8> for Direction {
    fn from(value: u8) -> Self {
        match value {
            1 => Direction::North,
            2 => Direction::South,
            3 => Self::East,
            4 => Self::West,
            5 => Self::NoEast,
            6 => Self::NoWest,
            7 => Direction::SoEast,
            8 => Direction::SoWest,
            _ => unreachable!(),
        }
    }
}
