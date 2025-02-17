fn main() {
    // Tests the PPM format.

    const IMAGE_WIDTH: u32 = 256;
    const IMAGE_HEIGHT: u32 = 256;

    print!("P3\n{} {}\n255\n", IMAGE_WIDTH, IMAGE_HEIGHT);

    for j in 0..IMAGE_HEIGHT {
        for i in 0..IMAGE_WIDTH {
            let r = (i as f64) / (IMAGE_WIDTH as f64 - 1.0);
            let g = (j as f64) / (IMAGE_HEIGHT as f64 - 1.0);
            let b = 0.0;

            let r = (255.0 * r) as u32;
            let g = (255.0 * g) as u32;
            let b = (255.0 * b) as u32;

            print!("{} {} {}\n", r, g, b);
        }
    }
}
