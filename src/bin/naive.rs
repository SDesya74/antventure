use std::{fmt::Debug, path::Path, process::ExitCode};

use image::{GrayImage, Luma};

const MAP_SIZE: usize = 1024;

fn main() -> ExitCode {
    let mut map = Map::new();

    let mut ant = Ant::new(
        Pos::new(MAP_SIZE as isize / 2, MAP_SIZE as isize / 2),
        Direction::North,
    );

    ant.walk_until_end(&mut map);

    println!("Ant leaved map at {:?}, looking at {:?}", ant.pos, ant.dir);

    println!("Black tiles count: {}", map.count_black_tiles());
    save_map_to_file(&map, "ant.png").expect("Error in saving");

    ExitCode::SUCCESS
}

fn save_map_to_file(map: &Map, file: impl AsRef<Path>) -> Result<(), image::ImageError> {
    let mut img = GrayImage::new(MAP_SIZE as _, MAP_SIZE as _);

    for y in 0..MAP_SIZE as _ {
        for x in 0..MAP_SIZE as _ {
            let is_white = map.get(&Pos::new(x, y)).unwrap();
            img.put_pixel(x as _, y as _, Luma([if is_white { 255 } else { 0 }]));
        }
    }

    img.save(file)
}

struct Map([[bool; MAP_SIZE]; MAP_SIZE]);

impl Map {
    fn new() -> Self {
        Self([[true; MAP_SIZE]; MAP_SIZE])
    }

    fn get_mut<'m>(&'m mut self, pos: &Pos) -> Option<&'m mut bool> {
        self.0.get_mut(pos.y as usize)?.get_mut(pos.x as usize)
    }

    fn get(&self, pos: &Pos) -> Option<bool> {
        self.0.get(pos.y as usize)?.get(pos.x as usize).copied()
    }

    fn count_black_tiles(&self) -> usize {
        self.0.iter().flatten().filter(|e| !**e).count()
    }
}

#[derive(Debug)]
struct Ant {
    pos: Pos,
    dir: Direction,
}

impl Ant {
    fn new(pos: Pos, dir: Direction) -> Self {
        Self { pos, dir }
    }

    /// Returns whether the ant can walk any further
    fn walk(&mut self, map: &mut Map) -> bool {
        let cell = map.get_mut(&self.pos).unwrap();

        *cell = !*cell;

        self.dir = match cell {
            true => self.dir.cw(),
            false => self.dir.ccw(),
        };

        let shift = self.dir.to_shift();

        let new_x = self.pos.x + shift.x;
        let new_y = self.pos.y + shift.y;

        if new_x < 0 || new_x >= MAP_SIZE as _ || new_y < 0 || new_y >= MAP_SIZE as _ {
            return false;
        }

        self.pos.x = new_x;
        self.pos.y = new_y;

        true
    }

    fn walk_until_end(&mut self, map: &mut Map) {
        while self.walk(map) {}
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Direction {
    North = 0,
    East = 1,
    South = 2,
    West = 3,
}

impl Direction {
    const VARIANTS: [Direction; 4] = [
        Direction::North,
        Direction::East,
        Direction::South,
        Direction::West,
    ];

    /// Rotate clockwise
    fn cw(self) -> Self {
        Self::VARIANTS[(self as usize + 1) % Self::VARIANTS.len()]
    }

    /// Rotate counterclockwise
    fn ccw(self) -> Self {
        Self::VARIANTS[(self as isize - 1).rem_euclid(Self::VARIANTS.len() as _) as usize]
    }

    fn to_shift(self) -> Pos {
        match self {
            Direction::North => Pos::new(0, -1),
            Direction::East => Pos::new(1, 0),
            Direction::South => Pos::new(0, 1),
            Direction::West => Pos::new(-1, 0),
        }
    }
}

#[derive(Debug)]
struct Pos {
    x: isize,
    y: isize,
}

impl Pos {
    fn new(x: isize, y: isize) -> Self {
        Self { x, y }
    }
}

#[test]
fn test_directions() {
    assert_eq!(Direction::North.cw(), Direction::East);
    assert_eq!(Direction::North.ccw(), Direction::West);
    assert_eq!(Direction::North.cw().cw(), Direction::South);
    assert_eq!(Direction::North.ccw().ccw(), Direction::South);
}

#[test]
fn check_map_bounds() {
    let mut map = Map::new();

    let mut ant = Ant::new(Pos::new(0, 0), Direction::North);

    assert!(!ant.walk(&mut map)); // ant can't go any further
}
