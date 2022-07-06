use enum_iterator::all;
use rand::prelude::{IteratorRandom, SliceRandom};
use rand::thread_rng;
use std::cell::RefCell;
use std::iter::zip;
use std::rc::{Rc, Weak};

use enum_map::{enum_map, EnumMap};
use ndarray::Array2;

use crate::tile_data::{Matches, TileData};

use crate::direction::Direction;

type Neighbor = Option<Weak<Cell>>;

pub struct ContradictionError;

pub struct Cell {
    pub options: RefCell<Vec<Rc<TileData>>>,
    sides: EnumMap<Direction, RefCell<Neighbor>>,
}

impl Cell {
    pub fn new(initial_options: Vec<Rc<TileData>>) -> Self {
        Self {
            options: RefCell::new(initial_options),
            sides: enum_map! {_ => RefCell::new(None)},
        }
    }

    pub fn set_neigh(&self, dir: Direction, neigh: Weak<Cell>) {
        *self.sides[dir].borrow_mut() = Some(neigh);
    }

    pub fn collapsed_result(&self) -> Option<Rc<TileData>> {
        if self.collapsed() {
            Some(self.options.borrow().first().unwrap().clone())
        } else {
            None
        }
    }

    pub fn collapsed(&self) -> bool {
        self.options.borrow().len() == 1
    }

    pub fn entropy(&self) -> usize {
        self.options.borrow().len()
    }

    pub fn collapse_random(&self) -> Result<(), ContradictionError> {
        let mut rng = thread_rng();
        let value = self
            .options
            .borrow()
            .choose_weighted(&mut rng, |tile| tile.weight)
            .map_err(|_| ContradictionError)?
            .clone();
        *self.options.borrow_mut() = vec![value];
        self.notify_neighbors();
        Ok(())
    }

    fn notify_neighbors(&self) {
        for (_, neigh) in &self.sides {
            let neigh: &Option<_> = &neigh.borrow();
            if let Some(neigh) = neigh {
                neigh.upgrade().unwrap().recalculate_options();
            }
        }
    }
    pub fn recalculate_options(&self) {
        let initial_len = self.options.borrow().len();
        for side in all::<Direction>() {
            let cell: &Neighbor = &self.sides[side].borrow();
            let opposite_side = side.opposite();
            if let Some(cell) = cell {
                self.options.borrow_mut().retain(|opt| {
                    let socket = &opt.sockets[side];
                    cell.upgrade()
                        .unwrap()
                        .options
                        .borrow()
                        .iter()
                        .any(|other| socket.matches(&other.sockets[opposite_side]))
                })
            }
        }
        if self.options.borrow().len() != initial_len {
            self.notify_neighbors();
        }
    }
}

pub struct TileGrid {
    pub grid: Array2<Rc<Cell>>,
    tiles: Vec<Rc<TileData>>,
}

impl Clone for TileGrid {
    fn clone(&self) -> Self {
        let new = Self::new(&self.tiles, self.width(), self.height());
        for (cell, original) in zip(&new.grid, &self.grid) {
            *cell.options.borrow_mut() = original.options.borrow().clone();
        }
        new
    }
}

impl TileGrid {
    pub fn new(tiles: &Vec<Rc<TileData>>, width: usize, height: usize) -> Self {
        let g = Self {
            grid: Array2::from_shape_simple_fn([height, width], || {
                Rc::new(Cell::new(tiles.clone()))
            }),
            tiles: tiles.clone(),
        };
        for y in 0..height - 1 {
            for x in 0..width {
                g.grid[(y, x)].set_neigh(Direction::Down, Rc::downgrade(&g.grid[(y + 1, x)]));
                g.grid[(y + 1, x)].set_neigh(Direction::Up, Rc::downgrade(&g.grid[(y, x)]));
            }
        }
        for x in 0..width - 1 {
            for y in 0..height {
                g.grid[(y, x)].set_neigh(Direction::Right, Rc::downgrade(&g.grid[(y, x + 1)]));
                g.grid[(y, x + 1)].set_neigh(Direction::Left, Rc::downgrade(&g.grid[(y, x)]));
            }
        }
        g
    }

    pub fn width(&self) -> usize {
        self.grid.raw_dim()[1]
    }
    pub fn height(&self) -> usize {
        self.grid.raw_dim()[0]
    }

    pub fn collapse_lowest_entropy(&self) -> Result<(), ContradictionError> {
        if let Some(min_entropy) = self
            .grid
            .iter()
            .filter(|c| !c.collapsed())
            .map(|c| c.entropy())
            .min()
        {
            let min_values = self.grid.iter().filter(|c| c.entropy() == min_entropy);
            let mut rng = thread_rng();
            if let Some(random_cell) = min_values.choose(&mut rng) {
                random_cell.collapse_random()?;
            }
        }
        Ok(())
    }

    pub fn reset(&self) {
        for cell in &self.grid {
            let mut opt = cell.options.borrow_mut();
            *opt = self.tiles.clone();
        }
    }
}
