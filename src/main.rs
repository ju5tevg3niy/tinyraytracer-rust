use std::{
    fs::File,
    io::{BufWriter, Write},
};

fn main() {
    render();
}

fn render() {
    const WIDTH: usize = 1024;
    const HEIGHT: usize = 768;

    let mut framebuffer = Vec::new();
    framebuffer.resize(WIDTH * HEIGHT, (0.0, 0.0, 0.0));

    for j in 0..HEIGHT {
        for i in 0..WIDTH {
            framebuffer[i + j * WIDTH].0 = j as f64 / HEIGHT as f64;
            framebuffer[i + j * WIDTH].1 = i as f64 / WIDTH as f64;
        }
    }

    let out = File::create("out.ppm").expect("Failed to create file");

    let mut writer = BufWriter::new(out);

    write!(writer, "P6\n{} {}\n255\n", WIDTH, HEIGHT).expect("Failed to write file header");

    for pixel in framebuffer {
        let (r, g, b) = pixel;
        writer
            .write(&[(r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8])
            .expect("Failed to write pixel data");
    }
}
