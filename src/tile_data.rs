use crate::direction::Direction;
use crate::parse_json;
use cairo::{Context, Filter, Format, ImageSurface, SurfacePattern};
use enum_map::enum_map;
use enum_map::EnumMap;
use std::error::Error;
use std::f64::consts::PI;
use std::fs::File;
use std::path::Path;

fn load_image(name: &Path) -> Result<ImageSurface, Box<dyn Error>> {
    Ok(ImageSurface::create_from_png(&mut File::open(name)?)?)
}

pub type Socket = String;
pub trait Matches {
    fn matches(&self, other: &Self) -> bool;
}
impl Matches for Socket {
    fn matches(&self, other: &Self) -> bool {
        self.chars().eq(other.chars().rev())
    }
}
trait Rotatable: Sized + Clone {
    fn rotate(&self) -> Self;

    fn rotate_n(&self, n: u8) -> Self {
        (0..n).fold(self.clone(), |p, _| p.rotate())
    }
}
impl<T: Clone> Rotatable for EnumMap<Direction, T> {
    fn rotate(&self) -> Self {
        enum_map! {
            d => self[match d {
                Direction::Up => Direction::Left,
                Direction::Left => Direction::Down,
                Direction::Down => Direction::Right,
                Direction::Right => Direction::Up,
            }].clone()
        }
    }
}

pub struct TileData {
    pub image: ImageSurface,
    pub sockets: EnumMap<Direction, Socket>,
    pub weight: f64,
}

pub struct TileSetData {
    pub background: Option<ImageSurface>,
    pub tiles: Vec<TileData>,
}

fn transform_image(image: &ImageSurface, scale: i32, rotation: i8) -> Result<ImageSurface, Box<dyn Error>> {
    let new_rotation =
        ImageSurface::create(Format::ARgb32, image.width() * scale, image.height() * scale)?;
    {
        let ctx = Context::new(&new_rotation)?;
        let scale = scale as f64;
        let sp = SurfacePattern::create(&image);
        sp.set_filter(Filter::Nearest);

        ctx.scale(scale, scale);
        ctx.translate(image.width() as f64 / 2.0, image.height() as f64 / 2.0);
        ctx.rotate(rotation as f64 * PI / 2.0);
        ctx.translate(-image.width() as f64 / 2.0, -image.height() as f64 / 2.0);
        ctx.set_source(&sp)?;
        ctx.paint()?;
    }
    Ok(new_rotation)
}

pub fn load_all_tiles(dir: &Path, scale: i32) -> Result<TileSetData, Box<dyn Error>> {
    let mut tiles = vec![];

    let data = parse_json::parse_tiles(&File::open(&dir.join("tiles.json"))?)?;
    for tile in data.tiles {
        let img = load_image(&dir.join(tile.file))?;
        for rotation in tile.rotations {
            let image = transform_image(&img, scale, rotation)?;
            tiles.push(TileData {
                image,
                sockets: tile.sockets.rotate_n(rotation as u8),
                weight: tile.weight,
            });
        }
    }

    let background = data
        .background
        .map(|background| transform_image(&load_image(&dir.join(background)).unwrap(), scale, 0).unwrap());

    Ok(TileSetData { tiles, background })
}
