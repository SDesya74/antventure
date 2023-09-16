use std::{
    fmt::{Debug, Display},
    fs::File,
    io::BufWriter,
    marker::PhantomData,
    ops::Add,
    path::Path,
    process::ExitCode,
};

use boolvec::{BoolVec, RefBoolMut};
use png::{BitDepth, ColorType, Encoder, EncodingError};

const MAP_SIZE: usize = 1024;

fn main() -> ExitCode {
    let mut map = Map::<MAP_SIZE, MAP_SIZE>::new_white();

    let mut ant = Ant::new(
        &mut map,
        Pos::new(MAP_SIZE as isize / 2, MAP_SIZE as isize / 2),
        Direction::North,
    )
    .expect("Can't spawn ant on invalid position");

    ant.walk_until_end();

    println!("Ant leaved map at {}, looking at {:?}", ant.pos, ant.dir);

    println!("Black tiles count: {}", map.count_black_tiles());
    save_map_to_file(&map, "ant.png").expect("Error in saving");

    ExitCode::SUCCESS
}

fn save_map_to_file<const W: usize, const H: usize>(
    map: &Map<W, H>,
    file: impl AsRef<Path>,
) -> Result<(), EncodingError> {
    let file = File::create(file)?;
    let w = BufWriter::new(file);

    let mut encoder = Encoder::new(w, W as _, H as _);
    encoder.set_color(ColorType::Grayscale);
    encoder.set_depth(BitDepth::One);
    let mut writer = encoder.write_header()?;

    // BoolVec is, in fact, 1-bit grayscale representation in memory
    // At first I was manually merging 8 bools representing cell color into one u8,
    // but then I found BoolVec crate and used it for the sake of simplicity
    let bytes = map.0.bytes().copied().collect::<Vec<_>>();

    // We also can save allocation here by use some unsafe
    // because we know that first field of BoolVec is Vec<u8>
    // let bytes = unsafe {
    //     let addr = std::ptr::addr_of!(map.0) as *const Vec<u8>;
    //     &*addr
    // };

    writer.write_image_data(&bytes[0..(W * H / u8::BITS as usize)])
}

#[derive(Clone)]
pub struct CellMut<'m>(RefBoolMut<'m>);

impl<'m> CellMut<'m> {
    fn is_white(&self) -> bool {
        self.0.get()
    }

    fn invert(&mut self) {
        self.0.set(!self.0.get());
    }
}

struct Map<const W: usize, const H: usize>(BoolVec);

impl<const W: usize, const H: usize> Map<W, H> {
    fn new_white() -> Self {
        Self(BoolVec::filled_with(W * H, true))
    }

    fn get_mut<'m>(&'m mut self, pos: MapPos<'m, W, H>) -> CellMut<'m> {
        let i = pos.y * W + pos.x;
        // SAFETY: We know that i can't be out of bounds because MapPos is valid
        unsafe { CellMut(self.0.get_unchecked_mut(i)) }
    }

    fn count_black_tiles(&self) -> usize {
        self.0.count() - self.0.count_ones()
    }
}

// Ant has lifetime because he can mutate map and can't outlive it
struct Ant<'m, const W: usize, const H: usize> {
    map: &'m mut Map<W, H>,
    pos: MapPos<'m, W, H>,
    dir: Direction,
}

impl<'m, const W: usize, const H: usize> Ant<'m, W, H> {
    fn new(map: &'m mut Map<W, H>, pos: Pos, dir: Direction) -> Result<Self, Pos> {
        Ok(Self {
            pos: MapPos::validate_pos(pos)?,
            map,
            dir,
        })
    }

    /// Returns whether the ant can walk any further
    fn walk(&mut self) -> bool {
        let mut cell = self.map.get_mut(self.pos);
        cell.invert();

        self.dir = match cell.is_white() {
            true => self.dir.cw(),
            false => self.dir.ccw(),
        };

        let shift = self.dir.to_shift();

        let new_pos = self.pos + shift;

        let Ok(pos) = MapPos::validate_pos(new_pos) else {
            return false;
        };

        self.pos = pos;
        true
    }

    fn walk_until_end(&mut self) {
        while self.walk() {}
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

impl<'m, const W: usize, const H: usize> Add<Pos> for MapPos<'m, W, H> {
    type Output = Pos;

    fn add(self, rhs: Pos) -> Self::Output {
        Self::Output {
            x: self.x as isize + rhs.x,
            y: self.y as isize + rhs.y,
        }
    }
}

#[test]
fn test_directions() {
    assert_eq!(Direction::North.cw(), Direction::East);
    assert_eq!(Direction::North.ccw(), Direction::West);
    assert_eq!(Direction::North.cw().cw(), Direction::South);
    assert_eq!(Direction::North.ccw().ccw(), Direction::South);
}

/// A valid position on a [`Map`]
#[derive(Clone, Copy)]
struct MapPos<'m, const W: usize, const H: usize> {
    x: usize,
    y: usize,
    _p: PhantomData<&'m Map<W, H>>,
}

impl<'m, const W: usize, const H: usize> Display for MapPos<'m, W, H> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl<'m, const W: usize, const H: usize> MapPos<'m, W, H> {
    const fn validate_pos(pos: Pos) -> Result<Self, Pos> {
        if pos.x < 0 || pos.x >= W as _ || pos.y < 0 || pos.y >= H as _ {
            Err(pos)
        } else {
            Ok(Self {
                x: pos.x as _,
                y: pos.y as _,
                _p: PhantomData,
            })
        }
    }
}

#[test]
fn check_map_bounds() {
    let mut map = Map::<1, 1>::new_white();

    let mut ant = Ant::new(&mut map, Pos::new(0, 0), Direction::North)
        .expect("Can't spawn ant on invalid position");

    assert!(!ant.walk()); // ant can't go any further
}
