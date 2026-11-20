//! Mathematical primitives: BlockPos, Vec3d, BlockBox, Rotation, Mirror.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub const ORIGIN: BlockPos = BlockPos { x: 0, y: 0, z: 0 };

    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn add(&self, dx: i32, dy: i32, dz: i32) -> Self {
        Self {
            x: self.x + dx,
            y: self.y + dy,
            z: self.z + dz,
        }
    }

    pub fn offset(&self, dir: Direction) -> Self {
        let (dx, dy, dz) = dir.vector();
        self.add(dx, dy, dz)
    }

    pub fn offset_by(&self, dir: Direction, dist: i32) -> Self {
        let (dx, dy, dz) = dir.vector();
        self.add(dx * dist, dy * dist, dz * dist)
    }

    pub fn distance_sq(&self, other: &BlockPos) -> f64 {
        let dx = (self.x - other.x) as f64;
        let dy = (self.y - other.y) as f64;
        let dz = (self.z - other.z) as f64;
        dx * dx + dy * dy + dz * dz
    }

    pub fn to_long(&self) -> i64 {
        let x = (self.x as i64) & 0x3FFFFFF;
        let z = (self.z as i64) & 0x3FFFFFF;
        let y = (self.y as i64) & 0xFFF;
        (x << 38) | (z << 12) | y
    }

    pub fn from_long(val: i64) -> Self {
        let x = (val >> 38) as i32;
        let y = ((val << 52) >> 52) as i32;
        let z = ((val << 26) >> 38) as i32;
        Self { x, y, z }
    }
}

impl fmt::Display for BlockPos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}, {}, {}]", self.x, self.y, self.z)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

impl Direction {
    pub const ALL: [Direction; 6] = [
        Direction::Down,
        Direction::Up,
        Direction::North,
        Direction::South,
        Direction::West,
        Direction::East,
    ];

    pub fn vector(&self) -> (i32, i32, i32) {
        match self {
            Direction::Down => (0, -1, 0),
            Direction::Up => (0, 1, 0),
            Direction::North => (0, 0, -1),
            Direction::South => (0, 0, 1),
            Direction::West => (-1, 0, 0),
            Direction::East => (1, 0, 0),
        }
    }

    pub fn opposite(&self) -> Self {
        match self {
            Direction::Down => Direction::Up,
            Direction::Up => Direction::Down,
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::West => Direction::East,
            Direction::East => Direction::West,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec3d {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3d {
    pub const ZERO: Vec3d = Vec3d { x: 0.0, y: 0.0, z: 0.0 };

    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn add(&self, other: &Vec3d) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }

    pub fn sub(&self, other: &Vec3d) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }

    pub fn scale(&self, s: f64) -> Self {
        Self {
            x: self.x * s,
            y: self.y * s,
            z: self.z * s,
        }
    }

    pub fn length_sq(&self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn length(&self) -> f64 {
        self.length_sq().sqrt()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BlockBox {
    pub min_x: i32,
    pub min_y: i32,
    pub min_z: i32,
    pub max_x: i32,
    pub max_y: i32,
    pub max_z: i32,
}

impl BlockBox {
    pub fn new(min_x: i32, min_y: i32, min_z: i32, max_x: i32, max_y: i32, max_z: i32) -> Self {
        Self {
            min_x: min_x.min(max_x),
            min_y: min_y.min(max_y),
            min_z: min_z.min(max_z),
            max_x: min_x.max(max_x),
            max_y: min_y.max(max_y),
            max_z: min_z.max(max_z),
        }
    }

    pub fn from_corners(pos1: &BlockPos, pos2: &BlockPos) -> Self {
        Self::new(pos1.x, pos1.y, pos1.z, pos2.x, pos2.y, pos2.z)
    }

    pub fn from_origin_and_size(origin: &BlockPos, size: &BlockPos) -> Self {
        let end_x = origin.x + size.x - if size.x > 0 { 1 } else { -1 };
        let end_y = origin.y + size.y - if size.y > 0 { 1 } else { -1 };
        let end_z = origin.z + size.z - if size.z > 0 { 1 } else { -1 };
        Self::from_corners(origin, &BlockPos::new(end_x, end_y, end_z))
    }

    pub fn contains(&self, pos: &BlockPos) -> bool {
        pos.x >= self.min_x && pos.x <= self.max_x
            && pos.y >= self.min_y && pos.y <= self.max_y
            && pos.z >= self.min_z && pos.z <= self.max_z
    }

    pub fn size_x(&self) -> i32 {
        (self.max_x - self.min_x) + 1
    }

    pub fn size_y(&self) -> i32 {
        (self.max_y - self.min_y) + 1
    }

    pub fn size_z(&self) -> i32 {
        (self.max_z - self.min_z) + 1
    }

    pub fn volume(&self) -> u64 {
        (self.size_x() as u64) * (self.size_y() as u64) * (self.size_z() as u64)
    }

    pub fn min_pos(&self) -> BlockPos {
        BlockPos::new(self.min_x, self.min_y, self.min_z)
    }

    pub fn max_pos(&self) -> BlockPos {
        BlockPos::new(self.max_x, self.max_y, self.max_z)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Rotation {
    #[default]
    None,
    Clockwise90,
    Clockwise180,
    CounterClockwise90,
}

impl Rotation {
    pub fn rotate_pos(&self, pos: &BlockPos, max_x: i32, max_z: i32) -> BlockPos {
        match self {
            Rotation::None => *pos,
            Rotation::Clockwise90 => BlockPos::new(max_z - pos.z, pos.y, pos.x),
            Rotation::Clockwise180 => BlockPos::new(max_x - pos.x, pos.y, max_z - pos.z),
            Rotation::CounterClockwise90 => BlockPos::new(pos.z, pos.y, max_x - pos.x),
        }
    }

    pub fn rotate_direction(&self, dir: Direction) -> Direction {
        match self {
            Rotation::None => dir,
            Rotation::Clockwise90 => match dir {
                Direction::North => Direction::East,
                Direction::East => Direction::South,
                Direction::South => Direction::West,
                Direction::West => Direction::North,
                other => other,
            },
            Rotation::Clockwise180 => match dir {
                Direction::North => Direction::South,
                Direction::South => Direction::North,
                Direction::East => Direction::West,
                Direction::West => Direction::East,
                other => other,
            },
            Rotation::CounterClockwise90 => match dir {
                Direction::North => Direction::West,
                Direction::West => Direction::South,
                Direction::South => Direction::East,
                Direction::East => Direction::North,
                other => other,
            },
        }
    }

    pub fn next(&self) -> Self {
        match self {
            Rotation::None => Rotation::Clockwise90,
            Rotation::Clockwise90 => Rotation::Clockwise180,
            Rotation::Clockwise180 => Rotation::CounterClockwise90,
            Rotation::CounterClockwise90 => Rotation::None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mirror {
    #[default]
    None,
    LeftRight, // X-axis mirror
    FrontBack, // Z-axis mirror
}

impl Mirror {
    pub fn mirror_pos(&self, pos: &BlockPos, max_x: i32, max_z: i32) -> BlockPos {
        match self {
            Mirror::None => *pos,
            Mirror::LeftRight => BlockPos::new(max_x - pos.x, pos.y, pos.z),
            Mirror::FrontBack => BlockPos::new(pos.x, pos.y, max_z - pos.z),
        }
    }

    pub fn mirror_direction(&self, dir: Direction) -> Direction {
        match self {
            Mirror::None => dir,
            Mirror::LeftRight => match dir {
                Direction::West => Direction::East,
                Direction::East => Direction::West,
                other => other,
            },
            Mirror::FrontBack => match dir {
                Direction::North => Direction::South,
                Direction::South => Direction::North,
                other => other,
            },
        }
    }

    pub fn next(&self) -> Self {
        match self {
            Mirror::None => Mirror::LeftRight,
            Mirror::LeftRight => Mirror::FrontBack,
            Mirror::FrontBack => Mirror::None,
        }
    }
}
