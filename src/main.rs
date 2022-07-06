use gtk::gdk::ffi::*;
use gtk::gdk::keys::Key;
use gtk::gdk::EventKey;
use gtk::prelude::*;
use gtk::ApplicationWindow;
use gtk::DrawingArea;

use gtk::cairo::{Context, ImageSurface};
use std::iter::zip;
use std::path::Path;
use std::path::PathBuf;
use std::rc::Rc;
use tile_data::TileData;

pub mod direction;
pub mod parse_json;
pub mod tile_data;
pub mod tile_grid;
use tile_grid::TileGrid;

fn build_ui(
    application: &gtk::Application,
    dir: &Path,
    skip_draw: usize,
    width: usize,
    height: usize,
    scale: usize,
) {
    let mut tile_set =
        tile_data::load_all_tiles(dir, scale as i32).expect("Error while loading TileSet");
    let tiles = tile_set.tiles.drain(..).map(|t| Rc::new(t)).collect();
    let grid = Rc::new(TileGrid::new(&tiles, width, height));
    let reference = &tiles[0].image;
    let cell_size = (reference.width() as usize, reference.height() as usize);

    let width = (cell_size.0 * grid.width()) as i32;
    let height = (cell_size.1 * grid.height()) as i32;
    let final_image = ImageSurface::create(cairo::Format::Rgb24, width, height).unwrap();
    let image_ctx = Context::new(&final_image).unwrap();
    // Paint white background
    image_ctx.set_source_rgb(1.0, 1.0, 1.0);
    image_ctx.paint().unwrap();
    let background = tile_set.background;

    let draw_cell = Rc::new(move |x: usize, y: usize, options: &Vec<Rc<TileData>>| {
        let x = (x * cell_size.0) as f64;
        let y = (y * cell_size.1) as f64;
        if let Some(background) = &background {
            image_ctx.set_source_surface(&background, x, y).unwrap();
        } else {
            image_ctx.set_source_rgb(1.0, 1.0, 1.0);
        }
        image_ctx.rectangle(x, y, cell_size.0 as f64, cell_size.1 as f64);
        image_ctx.fill().unwrap();
        for tile in options {
            image_ctx.set_source_surface(&tile.image, x, y).unwrap();
            image_ctx
                .paint_with_alpha(1.0 / options.len() as f64)
                .unwrap();
        }
    });

    let redraw_all = {
        let draw_cell = draw_cell.clone();
        let grid = grid.clone();
        move || {
            for ((y, x), cell) in grid.grid.indexed_iter() {
                draw_cell(x, y, &cell.options.borrow());
            }
        }
    };

    redraw_all();

    drawable(
        application,
        width,
        height,
        {
            let grid = grid.clone();
            move |da, cr| {
                let prev_grid = (*grid).clone();
                for _ in 0..skip_draw {
                    if let Err(_) = grid.collapse_lowest_entropy() {
                        grid.reset();
                    }
                }

                // Draw collapsed tiles
                for (((y, x), cell), prev) in zip(grid.grid.indexed_iter(), prev_grid.grid) {
                    //if let Some(tile) = cell.collapsed_result() {
                    //if options.len() != 1 {
                    //continue;
                    //}
                    let options: &Vec<_> = &cell.options.borrow();
                    let prev_options: &Vec<_> = &prev.options.borrow();

                    // Only draw cells if something changed.
                    if !(options.len() == prev_options.len()
                        && zip(options, prev_options).all(|(a, b)| Rc::ptr_eq(a, b)))
                    {
                        draw_cell(x, y, options);
                    }
                }
                //cr.set_source_rgba(1.0, 1.0, 1.0, 0.0);

                cr.set_source_surface(&final_image, 0.0, 0.0).unwrap();
                cr.paint().unwrap();

                da.queue_draw();
                Inhibit(false)
            }
        },
        {
            let grid = grid.clone();
            move |_, key_event| {
                if key_event.keyval() == Key::from(GDK_KEY_r as u32) {
                    grid.reset();
                    redraw_all();
                }
                Inhibit(false)
            }
        },
    );
}

use clap::{Parser, ValueHint::DirPath};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, parse(from_os_str), value_hint = DirPath)]
    dir: PathBuf,
    #[clap(long, default_value_t = 1)]
    skip_draw: usize,
    #[clap(short, long, default_value_t = 20)]
    width: usize,
    #[clap(short, long, default_value_t = 20)]
    height: usize,
    #[clap(short = 'z', long, default_value_t = 1)]
    scale: usize,
}

fn main() {
    let application = gtk::Application::new(
        Some("com.github.alansartorio.wave_function_collapse"),
        Default::default(),
    );

    let args = Args::parse();
    let dir = Rc::new(args.dir);

    application.connect_activate({
        let dir = dir.clone();
        move |a| build_ui(a, &dir, args.skip_draw, args.width, args.height, args.scale)
    });

    application.run_with_args::<String>(&[]);
}

pub fn drawable<F, K>(
    application: &gtk::Application,
    width: i32,
    height: i32,
    draw_fn: F,
    key_press: K,
) where
    F: Fn(&DrawingArea, &Context) -> Inhibit + 'static,
    K: Fn(&ApplicationWindow, &EventKey) -> Inhibit + 'static,
{
    let window = ApplicationWindow::new(application);
    let drawing_area = DrawingArea::new();

    drawing_area.connect_draw(draw_fn);
    window.connect_key_press_event(key_press);
    //timeout_add(Duration::from_millis(16), move || {
    //drawing_area.queue_draw();
    //glib::Continue(true)
    //});

    window.set_default_size(width, height);

    window.add(&drawing_area);
    window.show_all();
}
