use glam::f32::Vec3;
use sdl2::event::Event;
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::f32::consts::PI;

const WIDTH: usize = 1024;
const HEIGHT: usize = 768;
const FOV: f32 = PI / 2.0;

#[derive(Clone, Copy)]
struct Material {
    albedo: Vec3,
    diffuse_color: Vec3,
    specular_exponent: f32,
}

impl Material {
    fn new(a: Vec3, c: Vec3, s: f32) -> Self {
        Material {
            albedo: a,
            diffuse_color: c,
            specular_exponent: s
        }
    }
}

struct Sphere {
    center: Vec3,
    radius: f32,
    material: Material,
}

impl Sphere {
    fn new(c: Vec3, r: f32, m: Material) -> Self {
        Sphere {
            center: c,
            radius: r,
            material: m
        }
    }

    fn ray_intersect(&self, orig: Vec3, dir: Vec3, t0: &mut f32) -> bool {
        let big_l: Vec3 = self.center - orig;
        let tca: f32 = big_l.dot(dir);
        let d2: f32 = big_l.dot(big_l) - (tca * tca);
        if d2 > (self.radius * self.radius) {
            return false;
        }
        let thc: f32 = f32::sqrt(self.radius*self.radius - d2);
        *t0 = tca - thc;
        let t1 = tca + thc;
        if *t0 < 0.0 { *t0 = t1 };
        if *t0 < 0.0 { return false; }
        return true;
    }
}

struct Light {
    position: Vec3,
    intensity: f32,
}

fn scene_intersect(orig: Vec3, dir: Vec3, spheres: &Vec<Sphere>, hit: &mut Vec3, big_n: &mut Vec3, material: &mut Material) -> bool {
    let mut sphere_dist: f32 = f32::MAX;
    for sphere in spheres.iter() {
        let mut dist: f32 = 0.0;
        if sphere.ray_intersect(orig, dir, &mut dist) && dist < sphere_dist {
            sphere_dist = dist;
            *hit = orig + dir * dist;
            *big_n = (*hit - sphere.center).normalize();
            *material = sphere.material;
        }
    }
    sphere_dist < 1000.0 
}

fn reflect(i: Vec3, n: Vec3) -> Vec3 {
    i - (n * 2.0) * (i.dot(n))
}

// glam doesn't have a `norm` function (or i couldn't find it)
fn norm(v: Vec3) -> f32 {
    f32::sqrt(v.x * v.x + v.y * v.y + v.z * v.z)
}

fn cast_ray(orig: Vec3, dir: Vec3, spheres: &Vec<Sphere>, lights: &Vec<Light>, depth: usize) -> Vec3 {
    let mut point = Vec3::ZERO;
    let mut big_n = Vec3::ZERO;
    let mut material = Material::new(Vec3::ZERO, Vec3::ZERO, 0.0);

    if depth > 4 || !scene_intersect(orig, dir, &spheres, &mut point, &mut big_n, &mut material) {
        return Vec3::new(0.2, 0.7, 0.8); // background material 
    }

    let reflect_dir = reflect(dir, big_n).normalize();
    let reflect_orig = if reflect_dir.dot(big_n) < 0.0 {
        point - big_n * 0.001
    } else {
        point + big_n * 0.001
    };
    let reflect_color = cast_ray(reflect_orig, reflect_dir, &spheres, lights, depth + 1);
    
    let mut diffuse_light_intensity: f32 = 0.0;
    let mut specular_light_intensity: f32 = 0.0;
    for light in lights.iter() {
        let light_dir: Vec3 = (light.position - point).normalize();
        
        
        let light_distance: f32 = norm(light.position - point);
        let _orig: Vec3 = if light_dir.dot(big_n) < 0.0 {
            point - big_n * 0.001
        } else {
            point + big_n * 0.001
        };
        let mut _point = Vec3::ZERO;
        let mut _big_n = Vec3::ZERO;
        let mut _material = Material::new(Vec3::ZERO, Vec3::ZERO, 0.0);
        if scene_intersect(_orig, light_dir, &spheres, &mut _point, &mut _big_n, &mut _material) && norm(_point - _orig) < light_distance {
            continue;
        }

        diffuse_light_intensity += light.intensity * f32::max(0.0, light_dir.dot(big_n));
        specular_light_intensity += f32::powf(f32::max(0.0, -reflect(-light_dir, big_n).dot(dir)), material.specular_exponent) * light.intensity;
    }
    return material.diffuse_color * diffuse_light_intensity * material.albedo[0] + Vec3::ONE * specular_light_intensity * material.albedo[1] + reflect_color * material.albedo[2];
}

fn render(image: &Vec<Vec3>) -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsys = sdl_context.video()?;
    let window = video_subsys
        .window("Rusty Ray Tracing", WIDTH as u32, HEIGHT as u32)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    for (i, pixel) in image.iter().enumerate() {
        let row = i / WIDTH;
        let col = i % WIDTH;
        canvas.pixel(
            col as i16,
            row as i16,
            Color::RGB(
                (pixel.x * 255.0) as u8,
                (pixel.y * 255.0) as u8,
                (pixel.z * 255.0) as u8
            ),
        )?;
    }
    canvas.present();

    let mut events = sdl_context.event_pump()?;

    'running: loop {
        for event in events.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }
    }

    Ok(())
}

fn main() {
    let ivory      = Material::new(Vec3::new(0.6,  0.3, 0.1), Vec3::new(0.4, 0.4, 0.3),   50.0);
    let red_rubber = Material::new(Vec3::new(0.9,  0.1, 0.0), Vec3::new(0.3, 0.1, 0.1),   10.0);
    let mirror     = Material::new(Vec3::new(0.0, 10.0, 0.8), Vec3::new(1.0, 1.0, 1.0), 1425.0);

    let spheres = vec![
        Sphere::new(Vec3::new(-3.0,  0.0, -16.0), 2.0, ivory),
        Sphere::new(Vec3::new(-1.0, -1.5, -12.0), 2.0, mirror),
        Sphere::new(Vec3::new( 1.5, -0.5, -18.0), 3.0, red_rubber),
        Sphere::new(Vec3::new( 7.0,  5.0, -18.0), 4.0, mirror),
    ];

    let lights = vec![
        Light { position: Vec3::new(-20.0, 20.0,  20.0), intensity: 1.5 },
        Light { position: Vec3::new( 30.0, 50.0, -25.0), intensity: 1.8 },
        Light { position: Vec3::new( 30.0, 20.0,  30.0), intensity: 1.7 },
    ];

    let mut image = vec![Vec3::ZERO; WIDTH * HEIGHT];

    let mut x: f32;
    let mut y: f32;
    let mut dir: Vec3;
    for row in 0..HEIGHT {
        for col in 0..WIDTH {
            x = (2.0 * (col as f32 + 0.5) / (WIDTH as f32) - 1.0) * (FOV / 2.0).tan() * (WIDTH as f32 / HEIGHT as f32);
            y = -(2.0 * (row as f32 + 0.5) / (HEIGHT as f32) - 1.0) * (FOV / 2.0).tan();
            dir = Vec3::new(x, y, -1.0).normalize();
            image[row * WIDTH + col] = cast_ray(Vec3::ZERO, dir, &spheres, &lights, 0);
        }
    }
    
    let _ = render(&image);
}
