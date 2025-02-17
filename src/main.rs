use raytracing::{color::write_color, Color};
use std::io;

fn main() {
    // Tests the PPM format.

    const IMAGE_WIDTH: u32 = 256;
    const IMAGE_HEIGHT: u32 = 256;

    print!("P3\n{} {}\n255\n", IMAGE_WIDTH, IMAGE_HEIGHT);

    let mut stdout = io::stdout().lock();
    for j in 0..IMAGE_HEIGHT {
        for i in 0..IMAGE_WIDTH {
            let r = (i as f64) / (IMAGE_WIDTH as f64 - 1.0);
            let g = (j as f64) / (IMAGE_HEIGHT as f64 - 1.0);
            let b = 0.0;

            let color = Color::new(r, g, b);
            write_color(&mut stdout, &color);
        }
    }
}
