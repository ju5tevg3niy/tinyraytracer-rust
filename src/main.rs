use std::{
    f64::consts::PI,
    fs::File,
    io::{BufWriter, Write},
};

const EPS: f64 = 1e-3;
const CAST_RAY_DEPTH: usize = 2;

#[derive(Debug, Clone, Copy)]
struct Pixel {
    r: f64,
    g: f64,
    b: f64,
}

impl Pixel {
    fn to_vec3(self) -> Vec3 {
        Vec3 {
            x: self.r,
            y: self.g,
            z: self.b,
        }
    }
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

    fn to_pixel(self) -> Pixel {
        Pixel {
            r: self.x,
            g: self.y,
            b: self.z,
        }
    }

    fn reflect(&self, normal: &Self) -> Self {
        self.sub(&normal.mul(2.0 * self.dot(normal)))
    }

    fn refract(&self, mut normal: Self, refractive_index: f64) -> Self {
        let mut cosi = -self.dot(&normal).clamp(-1.0, 1.0);
        let eta;
        if cosi < 0.0 {
            cosi = -cosi;
            normal = normal.mul(-1.0);
            eta = refractive_index;
        } else {
            eta = 1.0 / refractive_index;
        }
        let k = 1.0 - eta.powi(2) * (1.0 - cosi.powi(2));
        if k < 0.0 {
            Self {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            }
        } else {
            self.mul(eta).add(&normal.mul(eta * cosi - k.sqrt()))
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Material {
    // color of material
    diffuse_color: Vec3,

    // albedo[0] - diffuse reflection constant
    // albedo[1] - specular reflection constant
    // albedo[2] - reflectance ?
    // albedo[3] - refractance ?
    albedo: [f64; 4],

    // shininess constant
    specular_exponent: f64,

    // ?
    refractive_index: f64,
}

#[derive(Debug)]
struct Sphere {
    center: Vec3,
    radius: f64,
    material: Material,
}

#[derive(Debug)]
struct Light {
    position: Vec3,
    intensity: f64,
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

fn scene_intersect(
    orig: &Vec3,
    dir: &Vec3,
    spheres: &Vec<Sphere>,
) -> Option<(Vec3, Vec3, Material)> {
    let mut closest_sphere = None;
    for sphere in spheres {
        if let Some(distance) = sphere.ray_intersect(orig, dir) {
            let s = Some((distance, sphere));
            match closest_sphere {
                None => closest_sphere = s,
                Some((old_distance, _)) => {
                    if distance < old_distance {
                        closest_sphere = s
                    }
                }
            }
        }
    }

    match closest_sphere {
        None => None,
        Some((distance, sphere)) => {
            let hit = orig.add(&dir.mul(distance));
            let normal = (hit.sub(&sphere.center)).normalize();
            let material = sphere.material;
            Some((hit, normal, material))
        }
    }
}

fn cast_ray(
    depth: usize,
    orig: &Vec3,
    dir: &Vec3,
    spheres: &Vec<Sphere>,
    lights: &Vec<Light>,
) -> Pixel {
    const BACKGROUND_COLOR: Pixel = Pixel {
        r: 0.2,
        g: 0.7,
        b: 0.8,
    };

    if depth > CAST_RAY_DEPTH {
        BACKGROUND_COLOR
    } else {
        match scene_intersect(orig, dir, spheres) {
            //background
            None => BACKGROUND_COLOR,
            //sphere
            Some((hit, normal, material)) => {
                // calculate shading via Phong model

                // diffuse light intensity
                let mut dli = 0.0;
                // specular light intensity
                let mut sli = 0.0;

                // move hit point a little to not intersect object again
                let perturb = normal.mul(EPS);
                let hit_outside = hit.add(&perturb);
                let hit_inside = hit.sub(&perturb);
                let reflect_color = {
                    let reflect_dir = dir.reflect(&normal);
                    let reflect_orig = if reflect_dir.dot(&normal) < 0.0 {
                        // ray is reflected from inside the object
                        hit_inside
                    } else {
                        // ray is reflected from outside of the object
                        hit_outside
                    };
                    cast_ray(depth + 1, &reflect_orig, &reflect_dir, spheres, lights).to_vec3()
                };
                let refract_color = {
                    let refract_dir = dir.refract(normal, material.refractive_index);
                    let refract_orig = if refract_dir.dot(&normal) < 0.0 {
                        // ray is reflected from inside the object
                        hit_inside
                    } else {
                        // ray is reflected from outside of the object
                        hit_outside
                    };
                    cast_ray(depth + 1, &refract_orig, &refract_dir, spheres, lights).to_vec3()
                };

                for light in lights {
                    let light_vec = light.position.sub(&hit);
                    let light_dir = light_vec.normalize();

                    let light_normal_projection = light_dir.dot(&normal);

                    let shadow_orig = if light_normal_projection < 0.0 {
                        // light is "behind" the hit
                        hit_inside
                    } else {
                        // light is "in front" of the hit
                        hit_outside
                    };

                    if let Some((shadow_hit, _, _)) =
                        scene_intersect(&shadow_orig, &light_dir, spheres)
                    {
                        if light_vec.norm() > shadow_hit.sub(&shadow_orig).norm() {
                            continue;
                        }
                    }

                    dli += light.intensity * light_normal_projection.max(0.0);
                    sli += light.intensity
                        * light_dir
                            .reflect(&normal)
                            .dot(dir)
                            .max(0.0)
                            .powf(material.specular_exponent);
                }

                material
                    .diffuse_color
                    .mul(dli * material.albedo[0])
                    .add(
                        &Vec3 {
                            x: 1.0,
                            y: 1.0,
                            z: 1.0,
                        }
                        .mul(sli * material.albedo[1]),
                    )
                    .add(&reflect_color.mul(material.albedo[2]))
                    .add(&refract_color.mul(material.albedo[3]))
                    .to_pixel()
            }
        }
    }
}

fn render(spheres: &Vec<Sphere>, lights: &Vec<Light>) {
    let out = File::create("out.ppm").expect("Failed to create file");

    let mut writer = BufWriter::new(out);

    write!(writer, "P6\n{WIDTH} {HEIGHT}\n255\n").expect("Failed to write file header");

    const WIDTH: usize = 1024;
    const HEIGHT: usize = 768;
    // pi/3 => 180deg/3 = 60deg
    // pi/2 => 180deg/2 = 90deg
    const FOV: f64 = PI / 3.0;
    let screen_width = 2.0 * (FOV / 2.0).tan();

    let mut framebuffer = Vec::with_capacity(WIDTH * HEIGHT);

    for i in 0..WIDTH * HEIGHT {
        let x = (i % WIDTH) as f64 + 0.5 - WIDTH as f64 / 2.0;
        let y = -((i / WIDTH) as f64) - 0.5 + HEIGHT as f64 / 2.0;
        let z = HEIGHT as f64 / -screen_width;
        let dir = Vec3 { x, y, z }.normalize();
        framebuffer.push(cast_ray(0, &ORIGIN, &dir, spheres, lights));
    }

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

fn main() {
    let ivory = Material {
        diffuse_color: Vec3 {
            x: 0.4,
            y: 0.4,
            z: 0.3,
        },
        albedo: [0.6, 0.3, 0.1, 0.0],
        specular_exponent: 50.0,
        refractive_index: 1.0,
    };
    let glass = Material {
        diffuse_color: Vec3 {
            x: 0.6,
            y: 0.7,
            z: 0.8,
        },
        albedo: [0.0, 0.5, 0.1, 0.8],
        specular_exponent: 125.0,
        refractive_index: 1.5,
    };
    let red_rubber = Material {
        diffuse_color: Vec3 {
            x: 0.3,
            y: 0.1,
            z: 0.1,
        },
        albedo: [0.9, 0.1, 0.0, 0.0],
        specular_exponent: 10.0,
        refractive_index: 1.0,
    };
    let mirror = Material {
        diffuse_color: Vec3 {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        },
        albedo: [0.0, 10.0, 0.8, 0.0],
        specular_exponent: 1425.0,
        refractive_index: 1.0,
    };

    let spheres = vec![
        Sphere {
            center: Vec3 {
                x: -3.0,
                y: 0.0,
                z: -16.0,
            },
            radius: 2.0,
            material: ivory,
        },
        Sphere {
            center: Vec3 {
                x: -1.0,
                y: -1.5,
                z: -12.0,
            },
            radius: 2.0,
            material: glass,
        },
        Sphere {
            center: Vec3 {
                x: 1.5,
                y: -0.5,
                z: -18.0,
            },
            radius: 3.0,
            material: red_rubber,
        },
        Sphere {
            center: Vec3 {
                x: 7.0,
                y: 5.0,
                z: -18.0,
            },
            radius: 4.0,
            material: mirror,
        },
    ];

    let lights = vec![
        Light {
            position: Vec3 {
                x: -20.0,
                y: 20.0,
                z: 20.0,
            },
            intensity: 1.5,
        },
        Light {
            position: Vec3 {
                x: 30.0,
                y: 50.0,
                z: -25.0,
            },
            intensity: 1.8,
        },
        Light {
            position: Vec3 {
                x: 30.0,
                y: 20.0,
                z: 30.0,
            },
            intensity: 1.7,
        },
    ];

    // dbg!(&spheres, &lights);

    render(&spheres, &lights);
}
