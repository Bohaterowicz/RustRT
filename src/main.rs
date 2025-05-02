mod aabb;
mod bvh;
mod camera;
mod entities;
mod interval;
mod material;
mod math;
mod mesh;
mod obj_reader;
mod pdf;
mod perlin_noise;
mod ray;
mod texture;
mod window;

use std::collections::VecDeque;
use std::fs::File;
use std::io::{self, Write};
use std::ops::Deref;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

use bvh::BVH;
use camera::Camera;
use entities::entity::Transformable;
use entities::mesh::Object;
use entities::quad::create_box;
use indicatif::ProgressBar;

use entities::{constant_medium::ConstantMedium, entity::EntityList, quad::Quad, sphere::Sphere};
use interval::Interval;
use material::*;
use math::vec3::*;
use obj_reader::read_obj;
use rand::seq::SliceRandom;
use ray::Ray;
use texture::{ImageTexture, NoiseTexture, Texture};
use window::Window;

#[derive(Debug, Default)]
pub struct Bitmap {
    width: i32,
    height: i32,
    data: Option<Vec<u8>>,
}

fn linear_to_gamma(value: f32) -> f32 {
    if value > 0.0 {
        return value.sqrt();
    }
    0.0
}

pub fn create_bitmap(width: i32, height: i32) -> Bitmap {
    let buffer_size = (width * height * 4) as usize;
    let data = Some(vec![0u8; buffer_size]);
    Bitmap {
        width,
        height,
        data,
    }
}

fn write_ppm(bitmap: &Bitmap) -> io::Result<()> {
    println!("Writing PPM file...");
    let mut file = File::create("render.ppm")?;
    writeln!(file, "P3")?;
    writeln!(file, "{} {}", bitmap.width, bitmap.height)?;
    writeln!(file, "255")?;
    let pb = ProgressBar::new((bitmap.height * bitmap.width) as u64);
    for y in 0..bitmap.height {
        for x in 0..bitmap.width {
            let offset = (y * bitmap.width + x) * 4;
            let r = bitmap.data.as_ref().unwrap()[offset as usize + 2];
            let g = bitmap.data.as_ref().unwrap()[offset as usize + 1];
            let b = bitmap.data.as_ref().unwrap()[offset as usize];
            writeln!(file, "{} {} {}", r, g, b)?;
            pb.inc(1);
        }
    }
    Ok(())
}

fn compute_chunks(total: u32, scale: u32) -> VecDeque<(u32, u32)> {
    let cache_line_size = 64;
    let chunk_count = total.div_ceil(cache_line_size * scale);
    let mut chunks = VecDeque::new();
    for i in 0..chunk_count {
        let start = i * cache_line_size * scale;
        let end = if i == chunk_count - 1 {
            total
        } else {
            (i + 1) * cache_line_size * scale
        };
        chunks.push_back((start, end));
    }
    // randomize the order of chunks
    let mut rng = rand::thread_rng();
    let mut chunk_vec: Vec<(u32, u32)> = chunks.iter().cloned().collect();
    chunk_vec.shuffle(&mut rng);
    chunks.clear();
    for chunk in chunk_vec {
        chunks.push_back(chunk);
    }
    chunks
}

fn render(
    threads: &mut Vec<thread::JoinHandle<()>>,
    count: u32,
    bitmap: &Arc<Mutex<Bitmap>>,
    image_width: u32,
    image_height: u32,
    entities: &Arc<EntityList>,
    lights: &Arc<EntityList>,
    camera: &Camera,
    stop: &Arc<AtomicBool>,
) {
    let thread_count = count;
    let pb = ProgressBar::new((image_height * image_width) as u64);
    let pb = Arc::new(Mutex::new(pb));
    let stdout = Arc::new(Mutex::new(io::stdout()));
    let chunks = Arc::new(Mutex::new(compute_chunks(
        image_width * image_height * 4,
        256,
    )));
    for _ in 0..thread_count {
        let pb_clone = Arc::clone(&pb);
        let buffer = Arc::clone(bitmap);
        let entities = Arc::clone(entities);
        let lights = Arc::clone(lights);
        let camera = camera.clone();
        let stdout = Arc::clone(&stdout);
        let chunks = Arc::clone(&chunks);
        let stop = Arc::clone(stop);
        let thread = thread::spawn(move || {
            let data: *mut u8 = {
                let mut buffer = buffer.lock().unwrap();
                buffer.data.as_mut().unwrap().as_mut_ptr()
            };

            loop {
                let (start, end) = {
                    let mut chunks = chunks.lock().unwrap();
                    if chunks.is_empty() {
                        break;
                    }
                    chunks.pop_front().unwrap()
                };
                {
                    let mut stdout = stdout.lock().unwrap();
                    if let Err(e) = writeln!(
                        stdout,
                        "Thread {:?} - Buffer size: {}",
                        thread::current().id(),
                        end - start
                    ) {
                        eprintln!("Error writing to stdout: {}", e);
                    }
                }
                for offset in (start..end).step_by(4) {
                    if stop.load(Ordering::Acquire) {
                        return;
                    }
                    let x = (offset / 4) % image_width;
                    let y = (offset / 4) / image_width;

                    let mut color = Vec3::zero();
                    for i in 0..camera.sqrt_spp {
                        for j in 0..camera.sqrt_spp {
                            let ray = camera.get_ray(x, y, i, j);
                            let mut ray_color = camera.ray_color(&ray, &entities, &lights, 0);
                            if ray_color.x.is_nan() {
                                ray_color.x = 0.0;
                            }
                            if ray_color.y.is_nan() {
                                ray_color.y = 0.0;
                            }
                            if ray_color.z.is_nan() {
                                ray_color.z = 0.0;
                            }
                            color += ray_color;
                        }
                    }
                    color *= camera.pixel_samples_scale;

                    let intensity = Interval::new(0.0, 0.999);
                    let ir = (255.99 * intensity.clamp(linear_to_gamma(color.x))) as u8;
                    let ig = (255.99 * intensity.clamp(linear_to_gamma(color.y))) as u8;
                    let ib = (255.99 * intensity.clamp(linear_to_gamma(color.z))) as u8;

                    unsafe {
                        data.add(offset as usize).write(ib);
                        data.add((offset + 1) as usize).write(ig);
                        data.add((offset + 2) as usize).write(ir);
                        data.add((offset + 3) as usize).write(0xFF);
                    }
                    let pb = pb_clone.lock().unwrap();
                    pb.inc(1);
                }
            }
        });
        threads.push(thread);
    }
}
/*
fn scene_scattered_balls(
    entities_out: &mut EntityList,
    camera: &mut Camera,
    width: u32,
    height: u32,
) {
    let mut entities = EntityList::new();

    let new_camera = Camera::new(
        width,
        height,
        20.0,
        &Vec3::new(13.0, 2.0, 3.0),
        &Vec3::zero(),
    );

    *camera = new_camera;

    let material_ground: Arc<dyn Material> = Arc::new(Lambertian {
        albedo: Box::new(CheckerTexture::new(
            Texture::new(Vec3::new(0.2, 0.3, 0.1)),
            Texture::new(Vec3::new(0.9, 0.9, 0.9)),
            0.32,
        )),
    });
    let material_1: Arc<dyn Material> = Arc::new(Dielectric {
        refraction_index: 1.5,
    });
    let material_2: Arc<dyn Material> = Arc::new(Lambertian {
        albedo: Box::new(Texture::new(Vec3::new(0.4, 0.2, 0.1))),
    });
    let material_3: Arc<dyn Material> = Arc::new(Metal {
        albedo: Vec3::new(0.7, 0.6, 0.5),
        fuzz: 0.0,
    });

    let center = Vec3::new(0.0, -1000.0, -1.0);
    let radius = 1000.0;
    entities.add(Box::new(Sphere::new(
        center,
        radius,
        Arc::clone(&material_ground),
    )));

    for a in -11..11 {
        for b in -11..11 {
            let choose_mat = rand_f32();
            let center = Vec3::new(
                (a as f32) + 0.9 * rand_f32(),
                0.2,
                (b as f32) * 0.9 * rand_f32(),
            );
            if (center - Vec3::new(4.0, 0.2, 0.0)).length() > 0.9 {
                if choose_mat < 0.8 {
                    let albedo = Vec3::random() * Vec3::random();
                    let material: Arc<dyn Material> = Arc::new(Lambertian {
                        albedo: Box::new(Texture::new(albedo)),
                    });
                    entities.add(Box::new(Sphere::new(center, 0.2, material)));
                } else if choose_mat < 0.95 {
                    let albedo = Vec3::random_range(0.5, 1.0);
                    let fuzz = rand_f32_range(0.0, 0.5);
                    let material: Arc<dyn Material> = Arc::new(Metal { albedo, fuzz });
                    entities.add(Box::new(Sphere::new(center, 0.2, material)));
                } else {
                    let material: Arc<dyn Material> = Arc::new(Dielectric {
                        refraction_index: 1.5,
                    });
                    entities.add(Box::new(Sphere::new(center, 0.2, material)));
                }
            }
        }
    }

    let center = Vec3::new(0.0, 1.0, 0.0);
    let radius = 1.0;
    entities.add(Box::new(Sphere::new(
        center,
        radius,
        Arc::clone(&material_1),
    )));

    let center = Vec3::new(-4.0, 1.0, 0.0);
    let radius = 0.5;
    entities.add(Box::new(Sphere::new(
        center,
        radius,
        Arc::clone(&material_2),
    )));

    let center = Vec3::new(4.0, 1.0, 0.0);
    let radius = 0.4;
    entities.add(Box::new(Sphere::new(
        center,
        radius,
        Arc::clone(&material_3),
    )));

    let bvh = BVH::new(entities);
    entities_out.add(Box::new(bvh));
}

fn checker_spheres(entities_out: &mut EntityList, camera: &mut Camera, width: u32, height: u32) {
    let mut entities = EntityList::new();

    let new_camera = Camera::new(
        width,
        height,
        20.0,
        &Vec3::new(13.0, 2.0, 3.0),
        &Vec3::zero(),
    );

    *camera = new_camera;

    let material_ground: Arc<dyn Material> = Arc::new(Lambertian {
        albedo: Box::new(CheckerTexture::new(
            Texture::new(Vec3::new(0.2, 0.3, 0.1)),
            Texture::new(Vec3::new(0.9, 0.9, 0.9)),
            0.32,
        )),
    });

    entities.add(Box::new(Sphere::new(
        Vec3::new(0.0, -10.0, 0.0),
        10.0,
        Arc::clone(&material_ground),
    )));

    entities.add(Box::new(Sphere::new(
        Vec3::new(0.0, 10.0, 0.0),
        10.0,
        Arc::clone(&material_ground),
    )));

    *entities_out = entities;
}
     */

fn scene_earth(entities_out: &mut EntityList, camera: &mut Camera, width: u32, height: u32) {
    let new_camera = Camera::new(
        width,
        height,
        20.0,
        &Vec3::new(0.0, 0.0, 12.0),
        &Vec3::zero(),
    );

    *camera = new_camera;

    let earth_material: Arc<dyn Material> = Arc::new(Lambertian {
        albedo: Box::new(ImageTexture::new("assets/earth.jpg")),
    });

    entities_out.add(Box::new(Sphere::new(
        Vec3::zero(),
        2.0,
        Arc::clone(&earth_material),
    )));
}

fn scene_perlin_spheres(
    entities_out: &mut EntityList,
    camera: &mut Camera,
    width: u32,
    height: u32,
) {
    let new_camera = Camera::new(
        width,
        height,
        20.0,
        &Vec3::new(13.0, 2.0, 3.0),
        &Vec3::zero(),
    );

    *camera = new_camera;

    let perlin_material: Arc<dyn Material> = Arc::new(Lambertian {
        albedo: Box::new(NoiseTexture::new(4.0)),
    });

    entities_out.add(Box::new(Sphere::new(
        Vec3::new(0.0, 2.0, 0.0),
        2.0,
        Arc::clone(&perlin_material),
    )));

    entities_out.add(Box::new(Sphere::new(
        Vec3::new(0.0, -1000.0, 0.0),
        1000.0,
        Arc::clone(&perlin_material),
    )));
}

fn scene_quads(entities_out: &mut EntityList, camera: &mut Camera, width: u32, height: u32) {
    let mut new_camera = Camera::new(
        width,
        height,
        80.0,
        &Vec3::new(0.0, 0.0, 9.0),
        &Vec3::zero(),
    );

    new_camera.set_background_color(&Vec3::new(0.70, 0.80, 1.00));

    *camera = new_camera;

    let left_red: Arc<dyn Material> = Arc::new(Lambertian {
        albedo: Box::new(Texture::new(Vec3::new(1.0, 0.2, 0.2))),
    });
    let back_green: Arc<dyn Material> = Arc::new(Lambertian {
        albedo: Box::new(Texture::new(Vec3::new(0.2, 1.0, 0.2))),
    });
    let right_blue: Arc<dyn Material> = Arc::new(Lambertian {
        albedo: Box::new(Texture::new(Vec3::new(0.2, 0.2, 1.0))),
    });
    let upper_orange: Arc<dyn Material> = Arc::new(Lambertian {
        albedo: Box::new(Texture::new(Vec3::new(1.0, 0.5, 0.0))),
    });
    let lower_teal: Arc<dyn Material> = Arc::new(Lambertian {
        albedo: Box::new(Texture::new(Vec3::new(0.2, 0.8, 0.8))),
    });

    entities_out.add(Box::new(Quad::new(
        Vec3::new(-3.0, -2.0, 5.0),
        Vec3::new(0.0, 0.0, -4.0),
        Vec3::new(0.0, 4.0, 0.0),
        Arc::clone(&left_red),
    )));
    entities_out.add(Box::new(Quad::new(
        Vec3::new(-2.0, -2.0, 0.0),
        Vec3::new(4.0, 0.0, -0.0),
        Vec3::new(0.0, 4.0, 0.0),
        Arc::clone(&back_green),
    )));
    entities_out.add(Box::new(Quad::new(
        Vec3::new(3.0, -2.0, 1.0),
        Vec3::new(0.0, 0.0, 4.0),
        Vec3::new(0.0, 4.0, 0.0),
        Arc::clone(&right_blue),
    )));
    entities_out.add(Box::new(Quad::new(
        Vec3::new(-2.0, 3.0, 1.0),
        Vec3::new(4.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 4.0),
        Arc::clone(&upper_orange),
    )));
    entities_out.add(Box::new(Quad::new(
        Vec3::new(-2.0, -3.0, 5.0),
        Vec3::new(4.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -4.0),
        Arc::clone(&lower_teal),
    )));
}

fn scene_simple_light(entities_out: &mut EntityList, camera: &mut Camera, width: u32, height: u32) {
    let new_camera = Camera::new(
        width,
        height,
        20.0,
        &Vec3::new(26.0, 3.0, 6.0),
        &Vec3::new(0.0, 2.0, 0.0),
    );

    *camera = new_camera;

    let perlin_material: Arc<dyn Material> = Arc::new(Lambertian {
        albedo: Box::new(NoiseTexture::new(4.0)),
    });

    entities_out.add(Box::new(Sphere::new(
        Vec3::new(0.0, 2.0, 0.0),
        2.0,
        Arc::clone(&perlin_material),
    )));

    entities_out.add(Box::new(Sphere::new(
        Vec3::new(0.0, -1000.0, 0.0),
        1000.0,
        Arc::clone(&perlin_material),
    )));

    let light_material: Arc<dyn Material> = Arc::new(DiffuseLight {
        emit: Box::new(Texture::new(Vec3::new(4.0, 4.0, 4.0))),
    });

    entities_out.add(Box::new(Quad::new(
        Vec3::new(3.0, 1.0, -2.0),
        Vec3::new(2.0, 0.0, 0.0),
        Vec3::new(0.0, 2.0, 0.0),
        Arc::clone(&light_material),
    )));
}

fn scene_cornell_box(
    entities_out: &mut EntityList,
    lights: &mut EntityList,
    camera: &mut Camera,
    width: u32,
    height: u32,
) {
    let new_camera = Camera::new(
        width,
        height,
        40.0,
        &Vec3::new(278.0, 278.0, -800.0),
        &Vec3::new(278.0, 278.0, 0.0),
    );

    *camera = new_camera;

    let red_material: Arc<dyn Material> = Arc::new(Lambertian {
        albedo: Box::new(Texture::new(Vec3::new(0.65, 0.05, 0.05))),
    });
    let white_material: Arc<dyn Material> = Arc::new(Lambertian {
        albedo: Box::new(Texture::new(Vec3::new(0.73, 0.73, 0.73))),
    });
    let green_material: Arc<dyn Material> = Arc::new(Lambertian {
        albedo: Box::new(Texture::new(Vec3::new(0.12, 0.45, 0.15))),
    });
    let yellow_material: Arc<dyn Material> = Arc::new(Lambertian {
        albedo: Box::new(Texture::new(Vec3::new(0.33, 0.33, 0.33))),
    });
    let light_material: Arc<dyn Material> = Arc::new(DiffuseLight {
        emit: Box::new(Texture::new(Vec3::new(15.0, 15.0, 15.0))),
    });
    let aluminium_material: Arc<dyn Material> = Arc::new(Metal {
        albedo: Vec3::new(0.8, 0.85, 0.88),
        fuzz: 0.0,
    });
    let glass: Arc<dyn Material> = Arc::new(Dielectric {
        refraction_index: 1.5,
    });

    entities_out.add(Box::new(Quad::new(
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        Arc::clone(&green_material),
    )));
    entities_out.add(Box::new(Quad::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        Arc::clone(&red_material),
    )));
    entities_out.add(Box::new(Quad::new(
        Vec3::new(0.0, 0.0, 555.0),
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        Arc::clone(&white_material),
    )));
    entities_out.add(Box::new(Quad::new(
        Vec3::new(555.0, 555.0, 555.0),
        Vec3::new(-555.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -555.0),
        Arc::clone(&white_material),
    )));
    entities_out.add(Box::new(Quad::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        Arc::clone(&yellow_material),
    )));

    let mut box1 = create_box(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(165.0, 330.0, 165.0),
        Arc::clone(&white_material),
    );
    box1.rotate(Vec3::new(0.0, 1.0, 0.0), 15.0);
    box1.translate(Vec3::new(265.0, 0.0, 295.0));
    entities_out.add(Box::from(box1));
    let mesh = read_obj("assets/mesh/cube.obj");
    let mut obj = Object::new(mesh, Arc::clone(&green_material));
    obj.translate(Vec3::new(50.0, 200.0, 300.0));
    entities_out.add(Box::from(obj));
    entities_out.add(Box::new(Sphere::new(
        Vec3::new(190.0, 90.0, 190.0),
        90.0,
        glass,
    )));

    entities_out.add(Box::new(Quad::new(
        Vec3::new(343.0, 554.0, 332.0),
        Vec3::new(-130.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -105.0),
        Arc::clone(&light_material),
    )));

    let empty_mat: Arc<dyn Material> = Arc::new(EmptyMaterial);
    lights.add(Box::new(Quad::new(
        Vec3::new(343.0, 554.0, 332.0),
        Vec3::new(-130.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -105.0),
        Arc::clone(&empty_mat),
    )));
    lights.add(Box::new(Sphere::new(
        Vec3::new(190.0, 90.0, 190.0),
        90.0,
        Arc::clone(&empty_mat),
    )));
}

fn scene_cornell_smoke(
    entities_out: &mut EntityList,
    camera: &mut Camera,
    width: u32,
    height: u32,
) {
    let new_camera = Camera::new(
        width,
        height,
        40.0,
        &Vec3::new(278.0, 278.0, -800.0),
        &Vec3::new(278.0, 278.0, 0.0),
    );

    *camera = new_camera;

    let red_material: Arc<dyn Material> = Arc::new(Lambertian {
        albedo: Box::new(Texture::new(Vec3::new(0.65, 0.05, 0.05))),
    });
    let white_material: Arc<dyn Material> = Arc::new(Lambertian {
        albedo: Box::new(Texture::new(Vec3::new(0.73, 0.73, 0.73))),
    });
    let green_material: Arc<dyn Material> = Arc::new(Lambertian {
        albedo: Box::new(Texture::new(Vec3::new(0.12, 0.45, 0.15))),
    });
    let light_material: Arc<dyn Material> = Arc::new(DiffuseLight {
        emit: Box::new(Texture::new(Vec3::new(15.0, 15.0, 15.0))),
    });

    entities_out.add(Box::new(Quad::new(
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        Arc::clone(&green_material),
    )));
    entities_out.add(Box::new(Quad::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        Arc::clone(&red_material),
    )));
    entities_out.add(Box::new(Quad::new(
        Vec3::new(343.0, 554.0, 332.0),
        Vec3::new(-130.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -105.0),
        Arc::clone(&light_material),
    )));
    entities_out.add(Box::new(Quad::new(
        Vec3::new(0.0, 0.0, 555.0),
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        Arc::clone(&white_material),
    )));
    entities_out.add(Box::new(Quad::new(
        Vec3::new(555.0, 555.0, 555.0),
        Vec3::new(-555.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -555.0),
        Arc::clone(&white_material),
    )));
    entities_out.add(Box::new(Quad::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        Arc::clone(&white_material),
    )));

    let black_smoke_material: Arc<dyn Material> = Arc::new(Isotropic {
        albedo: Box::new(Texture::new(Vec3::new(0.0, 0.0, 0.0))),
    });
    let white_smoke_material: Arc<dyn Material> = Arc::new(Isotropic {
        albedo: Box::new(Texture::new(Vec3::new(1.0, 1.0, 1.0))),
    });

    let mut box1 = create_box(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(165.0, 330.0, 165.0),
        Arc::clone(&white_material),
    );
    box1.rotate(Vec3::new(0.0, 1.0, 0.0), 15.0);
    box1.translate(Vec3::new(265.0, 0.0, 295.0));

    let mut box2 = create_box(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(165.0, 165.0, 165.0),
        Arc::clone(&white_material),
    );
    box2.rotate(Vec3::new(0.0, 1.0, 0.0), -18.0);
    box2.translate(Vec3::new(130.0, 0.0, 65.0));

    entities_out.add(Box::new(ConstantMedium::new(
        Box::from(box1),
        0.01,
        Arc::clone(&black_smoke_material),
    )));
    entities_out.add(Box::new(ConstantMedium::new(
        Box::from(box2),
        0.01,
        Arc::clone(&white_smoke_material),
    )));
}

fn scene_cornell_mesh(
    entities_out: &mut EntityList,
    lights: &mut EntityList,
    camera: &mut Camera,
    width: u32,
    height: u32,
) {
    let new_camera = Camera::new(
        width,
        height,
        40.0,
        &Vec3::new(278.0, 278.0, -800.0),
        &Vec3::new(278.0, 278.0, 0.0),
    );

    *camera = new_camera;

    let red_material: Arc<dyn Material> = Arc::new(Lambertian {
        albedo: Box::new(Texture::new(Vec3::new(0.65, 0.05, 0.05))),
    });
    let white_material: Arc<dyn Material> = Arc::new(Lambertian {
        albedo: Box::new(Texture::new(Vec3::new(0.73, 0.73, 0.73))),
    });
    let green_material: Arc<dyn Material> = Arc::new(Lambertian {
        albedo: Box::new(Texture::new(Vec3::new(0.12, 0.45, 0.15))),
    });
    let yellow_material: Arc<dyn Material> = Arc::new(Lambertian {
        albedo: Box::new(Texture::new(Vec3::new(0.33, 0.33, 0.33))),
    });
    let light_material: Arc<dyn Material> = Arc::new(DiffuseLight {
        emit: Box::new(Texture::new(Vec3::new(15.0, 15.0, 15.0))),
    });
    let glass: Arc<dyn Material> = Arc::new(Dielectric {
        refraction_index: 1.5,
    });

    entities_out.add(Box::new(Quad::new(
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        Arc::clone(&green_material),
    )));
    entities_out.add(Box::new(Quad::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        Arc::clone(&red_material),
    )));
    entities_out.add(Box::new(Quad::new(
        Vec3::new(0.0, 0.0, 555.0),
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        Arc::clone(&white_material),
    )));
    entities_out.add(Box::new(Quad::new(
        Vec3::new(555.0, 555.0, 555.0),
        Vec3::new(-555.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -555.0),
        Arc::clone(&white_material),
    )));
    entities_out.add(Box::new(Quad::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        Arc::clone(&yellow_material),
    )));

    let mesh = read_obj("assets/mesh/stanford-bunny.obj");
    let mut obj = Object::new(mesh, Arc::clone(&red_material));
    obj.rotate(Vec3::new(0.0, 1.0, 0.0), 180.0);
    obj.scale(Vec3::new(2000.0, 2000.0, 2000.0));
    obj.translate(Vec3::new(350.0, -70.0, 300.0));
    obj.bvh = Some(BVH::new(&obj.mesh));
    entities_out.add(Box::new(obj.clone()));

    let mesh = read_obj("assets/mesh/stanford-bunny.obj");
    let mut obj = Object::new(mesh, Arc::clone(&glass));
    obj.rotate(Vec3::new(0.0, 1.0, 0.0), 90.0);
    obj.scale(Vec3::new(1500.0, 1500.0, 1500.0));
    obj.translate(Vec3::new(150.0, -60.0, 100.0));
    obj.bvh = Some(BVH::new(&obj.mesh));
    entities_out.add(Box::new(obj.clone()));

    entities_out.add(Box::new(Quad::new(
        Vec3::new(343.0, 554.0, 332.0),
        Vec3::new(-130.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -105.0),
        Arc::clone(&light_material),
    )));

    let empty_mat: Arc<dyn Material> = Arc::new(EmptyMaterial);
    lights.add(Box::new(Quad::new(
        Vec3::new(343.0, 554.0, 332.0),
        Vec3::new(-130.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -105.0),
        Arc::clone(&empty_mat),
    )));

    lights.add(Box::new(Quad::new(
        Vec3::new(343.0, 554.0, 332.0),
        Vec3::new(-130.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -105.0),
        Arc::clone(&empty_mat),
    )));
}

fn scene_cornell_mesh_sphere(
    entities_out: &mut EntityList,
    lights: &mut EntityList,
    camera: &mut Camera,
    width: u32,
    height: u32,
) {
    let new_camera = Camera::new(
        width,
        height,
        40.0,
        &Vec3::new(278.0, 278.0, -800.0),
        &Vec3::new(278.0, 278.0, 0.0),
    );

    *camera = new_camera;

    let red_material: Arc<dyn Material> = Arc::new(Lambertian {
        albedo: Box::new(Texture::new(Vec3::new(0.65, 0.05, 0.05))),
    });
    let white_material: Arc<dyn Material> = Arc::new(Lambertian {
        albedo: Box::new(Texture::new(Vec3::new(0.73, 0.73, 0.73))),
    });
    let green_material: Arc<dyn Material> = Arc::new(Lambertian {
        albedo: Box::new(Texture::new(Vec3::new(0.12, 0.45, 0.15))),
    });
    let yellow_material: Arc<dyn Material> = Arc::new(Lambertian {
        albedo: Box::new(Texture::new(Vec3::new(0.33, 0.33, 0.33))),
    });
    let light_material: Arc<dyn Material> = Arc::new(DiffuseLight {
        emit: Box::new(Texture::new(Vec3::new(15.0, 15.0, 15.0))),
    });
    let glass: Arc<dyn Material> = Arc::new(Dielectric {
        refraction_index: 1.5,
    });

    entities_out.add(Box::new(Quad::new(
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        Arc::clone(&green_material),
    )));
    entities_out.add(Box::new(Quad::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        Arc::clone(&red_material),
    )));
    entities_out.add(Box::new(Quad::new(
        Vec3::new(0.0, 0.0, 555.0),
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        Arc::clone(&white_material),
    )));
    entities_out.add(Box::new(Quad::new(
        Vec3::new(555.0, 555.0, 555.0),
        Vec3::new(-555.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -555.0),
        Arc::clone(&white_material),
    )));
    entities_out.add(Box::new(Quad::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        Arc::clone(&yellow_material),
    )));

    let mesh = read_obj("assets/mesh/sphere_flat.obj");
    let mut obj = Object::new(mesh, Arc::clone(&red_material));
    //obj.rotate(Vec3::new(0.0, 1.0, 0.0), 180.0);
    obj.scale(Vec3::new(2.0, 2.0, 2.0));
    obj.translate(Vec3::new(250.0, 200.0, 350.0));
    obj.bvh = Some(BVH::new(&obj.mesh));
    entities_out.add(Box::new(obj.clone()));

    entities_out.add(Box::new(Quad::new(
        Vec3::new(343.0, 554.0, 332.0),
        Vec3::new(-130.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -105.0),
        Arc::clone(&light_material),
    )));

    let empty_mat: Arc<dyn Material> = Arc::new(EmptyMaterial);
    lights.add(Box::new(Quad::new(
        Vec3::new(343.0, 554.0, 332.0),
        Vec3::new(-130.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -105.0),
        Arc::clone(&empty_mat),
    )));

    lights.add(Box::new(Quad::new(
        Vec3::new(343.0, 554.0, 332.0),
        Vec3::new(-130.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -105.0),
        Arc::clone(&empty_mat),
    )));
}

fn main() {
    let mut use_ppm = true;
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|arg| arg == "--window") {
        use_ppm = false;
    }
    //let aspect_ratio = window.dim.width as f32 / window.dim.height as f32; //16f32/9f32;
    const DEFAULT_WIDTH: u32 = 1600;
    const DEFAULT_HEIGHT: u32 = 1200;
    let image_width = DEFAULT_WIDTH;
    let image_height = DEFAULT_HEIGHT;
    assert!(image_height > 1);
    let bitmap = create_bitmap(image_width as i32, image_height as i32);

    let mut entities = EntityList::new();
    let mut lights = EntityList::new();
    let mut camera = Camera::default();
    //scene_scattered_balls(&mut entities, &mut camera, image_width, image_height);
    //checker_spheres(&mut entities, &mut camera, image_width, image_height);
    //scene_earth(&mut entities, &mut camera, image_width, image_height);
    //scene_perlin_spheres(&mut entities, &mut camera, image_width, image_height);
    //scene_quads(&mut entities, &mut camera, image_width, image_height);
    //scene_simple_light(&mut entities, &mut camera, image_width, image_height);
    scene_cornell_mesh(
        &mut entities,
        &mut lights,
        &mut camera,
        image_width,
        image_height,
    );
    //scene_cornell_smoke(&mut entities, &mut camera, image_width, image_height);
    let entities = Arc::from(entities);
    let lights: Arc<EntityList> = Arc::from(lights);
    let thread_count = 24;
    let mut threads = Vec::with_capacity(thread_count as usize);
    let stop = Arc::new(AtomicBool::new(false));

    if use_ppm {
        let bitmap = Arc::new(Mutex::new(bitmap));
        render(
            &mut threads,
            thread_count,
            &bitmap,
            image_width,
            image_height,
            &entities,
            &lights,
            &camera,
            &stop,
        );
        for thread in threads {
            thread.join().unwrap();
        }
        println!("Rendering completed.");
        write_ppm(bitmap.lock().unwrap().deref()).unwrap();
        println!("PPM file written successfully.");
    } else {
        let window = Window::new("Raytracer", image_width as i32, image_height as i32, bitmap);
        let mut first = true;
        loop {
            window.process_messages();
            window.display();
            if first {
                render(
                    &mut threads,
                    thread_count,
                    &window.buffer.bitmap,
                    image_width,
                    image_height,
                    &entities,
                    &lights,
                    &camera,
                    &stop,
                );
                first = false;
            }
            if window.shutdown_requested {
                stop.store(true, Ordering::Release);
                for thread in threads {
                    thread.join().unwrap();
                }
                break;
            }
        }
    }
}
