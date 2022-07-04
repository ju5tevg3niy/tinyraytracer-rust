use std::{
    f64::consts::PI,
    fs::File,
    io::{BufWriter, Write},
};

#[derive(Debug, Clone, Copy)]
struct Pixel {
    r: f64,
    g: f64,
    b: f64,
}

#[derive(Debug, Clone, Copy)]
struct Vec3 {
    x: f64,
    y: f64,
    z: f64,
}

const ORIGIN: Vec3 = Vec3 {
    x: 0.0,
    y: 0.0,
    z: 0.0,
};

impl Vec3 {
    fn dot(&self, other: &Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    fn add(&self, other: &Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }

    fn sub(&self, other: &Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }

    fn mul(&self, val: f64) -> Self {
        Self {
            x: self.x * val,
            y: self.y * val,
            z: self.z * val,
        }
    }

    fn norm(&self) -> f64 {
        self.dot(self).sqrt()
    }

    fn normalize(&self) -> Self {
        self.mul(1.0 / self.norm())
    }
}

struct Sphere {
    center: Vec3,
    radius: f64,
}

impl Sphere {
    fn ray_intersect(&self, orig: &Vec3, dir: &Vec3) -> Option<f64> {
        let l = self.center.sub(orig);
        let tca = l.dot(dir);
        let d2 = l.dot(&l) - tca * tca;
        let r2 = self.radius * self.radius;
        if d2 > r2 {
            None
        } else {
            let thc = (r2 - d2).sqrt();
            let t0 = tca - thc;
            let t1 = tca + thc;

            if t0 >= 0.0 {
                Some(t0)
            } else if t1 >= 0.0 {
                Some(t1)
            } else {
                None
            }
        }
    }
}

fn main() {
    let s = Sphere {
        center: Vec3 {
            x: -3.0,
            y: 0.0,
            z: -16.0,
        },
        radius: 2.0,
    };

    render(&s);
}

fn cast_ray(orig: &Vec3, dir: &Vec3, sphere: &Sphere) -> Pixel {
    match sphere.ray_intersect(orig, dir) {
        //sphere color
        Some(_) => Pixel {
            r: 0.4,
            g: 0.4,
            b: 0.3,
        },
        //background color
        None => Pixel {
            r: 0.2,
            g: 0.7,
            b: 0.8,
        },
    }
}

fn render(sphere: &Sphere) {
    const WIDTH: usize = 1024;
    const HEIGHT: usize = 768;
    const FOV: f64 = PI / 2.0;
    let screen_width = 2.0 * (FOV / 2.0).tan();

    let mut framebuffer = Vec::with_capacity(WIDTH * HEIGHT);

    for i in 0..WIDTH * HEIGHT {
        let x = (i % WIDTH) as f64 + 0.5 - WIDTH as f64 / 2.0;
        let y = -((i / WIDTH) as f64) - 0.5 + HEIGHT as f64 / 2.0;
        let z = WIDTH as f64 / -screen_width;
        let dir = Vec3 { x, y, z }.normalize();
        framebuffer.push(cast_ray(&ORIGIN, &dir, sphere));
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
