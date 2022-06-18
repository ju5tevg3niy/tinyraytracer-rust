use std::{
    fs::File,
    io::{BufWriter, Write},
};

#[derive(Debug, Clone, Copy)]
struct Pixel {
    r: f64,
    g: f64,
    b: f64,
}

fn main() {
    render();
}

fn render() {
    const WIDTH: usize = 1024;
    const HEIGHT: usize = 768;

    let mut framebuffer = Vec::with_capacity(WIDTH * HEIGHT);

    for i in 0..WIDTH * HEIGHT {
        let x = i % WIDTH;
        let y = i / WIDTH;
        framebuffer.push(Pixel {
            r: x as f64 / WIDTH as f64,
            g: y as f64 / HEIGHT as f64,
            b: 0.0,
        });
    }

    let out = File::create("out.ppm").expect("Failed to create file");

    let mut writer = BufWriter::new(out);

    write!(writer, "P6\n{} {}\n255\n", WIDTH, HEIGHT).expect("Failed to write file header");

    for pixel in framebuffer {
        let pixel_bytes = [
            (pixel.r * 255.0) as u8,
            (pixel.g * 255.0) as u8,
            (pixel.b * 255.0) as u8,
        ];
        writer
            .write_all(&pixel_bytes)
            .expect("Failed to write pixel data");
    }
}
