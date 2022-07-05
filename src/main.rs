use gtk::prelude::*;
use gtk::DrawingArea;

use gtk::cairo::Context;

pub mod direction;
pub mod tile_data;
pub mod parse_json;

fn build_ui(application: &gtk::Application) {
    let tiles = tile_data::load_all_tiles().unwrap();

    drawable(application, 500, 500, move |_, cr| {
        cr.set_source_rgb(1.0, 1.0, 1.0);
        cr.paint().unwrap();
        let mut x = 0;
        for tile in &tiles {
            cr.set_source_surface(&tile.image, x as f64, 0.0).unwrap();
            cr.paint().unwrap();

            x += tile.image.width();
        }
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
    let drawing_area = Box::new(DrawingArea::new)();

    drawing_area.connect_draw(draw_fn);

    window.set_default_size(width, height);

    window.add(&drawing_area);
    window.show_all();
}
