use gtk::prelude::*;
use gtk::DrawingArea;

use gtk::cairo::{Context, ImageSurface};
use std::iter::zip;
use std::rc::Rc;
use tile_data::TileData;

pub mod direction;
pub mod parse_json;
pub mod tile_data;
pub mod tile_grid;
use tile_grid::TileGrid;

fn build_ui(application: &gtk::Application) {
    let tiles = tile_data::load_all_tiles(1)
        .unwrap()
        .drain(..)
        .map(|t| Rc::new(t))
        .collect();
    let size = 40;
    let grid = TileGrid::new(&tiles, size, size);
    let reference = &tiles[0].image;
    let size = (reference.width() as usize, reference.height() as usize);

    let width = (size.0 * grid.width()) as i32;
    let height = (size.1 * grid.height()) as i32;
    let final_image = ImageSurface::create(cairo::Format::Rgb24, width, height).unwrap();
    let image_ctx = Context::new(&final_image).unwrap();
    // Paint white background
    image_ctx.set_source_rgb(1.0, 1.0, 1.0);
    image_ctx.paint().unwrap();

    let draw_cell = move |x: usize, y: usize, options: &Vec<Rc<TileData>>| {
        let x = (x * size.0) as f64;
        let y = (y * size.1) as f64;
        image_ctx.set_source_rgb(1.0, 1.0, 1.0);
        image_ctx.rectangle(x, y, size.0 as f64, size.1 as f64);
        image_ctx.fill().unwrap();
        for tile in options {
            image_ctx.set_source_surface(&tile.image, x, y).unwrap();
            image_ctx
                .paint_with_alpha(1.0 / options.len() as f64)
                .unwrap();
        }
    };

    for ((y, x), cell) in grid.grid.indexed_iter() {
        draw_cell(x, y, &cell.options.borrow());
    }

    drawable(application, height, width, move |da, cr| {
        let prev_grid = grid.clone();
        for _ in 0..10 {
            grid.collapse_lowest_entropy();
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
    });
}

fn main() {
    let application = gtk::Application::new(
        Some("com.github.gtk-rs.examples.cairotest"),
        Default::default(),
    );

    application.connect_activate(build_ui);

    application.run();
}

pub fn drawable<F>(application: &gtk::Application, width: i32, height: i32, draw_fn: F)
where
    F: Fn(&DrawingArea, &Context) -> Inhibit + 'static,
{
    let window = gtk::ApplicationWindow::new(application);
    let drawing_area = DrawingArea::new();

    drawing_area.connect_draw(draw_fn);
    //timeout_add(Duration::from_millis(16), move || {
    //drawing_area.queue_draw();
    //glib::Continue(true)
    //});

    window.set_default_size(width, height);

    window.add(&drawing_area);
    window.show_all();
}
