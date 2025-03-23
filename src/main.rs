
mod math;
mod ray;
mod entities;
mod interval;
mod camera;
mod material;
mod window;

use std::fs::File;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use std::thread;

use camera::Camera;
use indicatif::ProgressBar;

use interval::Interval;
use math::rand::{rand_f32, rand_f32_range};
use math::vec3::*;
use ray::Ray;
use entities::{entity::EntityList, sphere::Sphere};
use material::*;
use window::{create_back_buffer, Window};

fn linear_to_gamma(value: f32) -> f32 {
    if value > 0.0 {
        return value.sqrt()
    }
    0.0
}

/* 
fn write_ppm(image: &mut Image, camera: &Camera, entity_list: &EntityList) ->io::Result<()> {
    let mut file = File::create("render.ppm")?;
    writeln!(file, "P3")?;
    writeln!(file, "{} {}", image.width, image.height)?;
    writeln!(file, "255")?;

    let pb = ProgressBar::new((image.height*image.width) as u64);
    for y in 0..image.height {
        for x in 0..image.width {
            let mut color = vec3(0.0, 0.0, 0.0);
            for _ in 0..camera.samples_per_pixel {
                let ray = camera.get_ray(x, y);
                color += camera.ray_color(&ray, entity_list, 0);
            } 
            color *= camera.pixel_samples_scale;
            let intensity = Interval::new(0.0, 0.999);
            let ir = (255.99 * intensity.clamp(linear_to_gamma(color.x))) as i32;
            let ig = (255.99 * intensity.clamp(linear_to_gamma(color.y))) as i32;
            let ib = (255.99 * intensity.clamp(linear_to_gamma(color.z))) as i32;
        
            writeln!(file, "{} {} {}", ir, ig, ib)?;
            pb.inc(1);
        }
    }
    pb.finish();
    Ok(())
}
*/

fn render(window: &Window, image_width: u32, image_height: u32, entities: &Arc<EntityList>, camera: &Camera) {
    let thread_count = 24;
    let pb = ProgressBar::new((image_height*image_width) as u64);
    let pb = Arc::new(Mutex::new(pb));
    let mut threads = Vec::with_capacity(thread_count as usize);
    let chunk_size = (image_width * image_height * 4) / thread_count;
    let stdout = Arc::new(Mutex::new(io::stdout()));
    for i in 0..thread_count {
        let pb_clone = Arc::clone(&pb);
        let buffer = Arc::clone(&window.buffer);
        let start = i * chunk_size;
        let end = if i == thread_count-1 {image_width * image_height * 4} else {(i+1) * chunk_size};
        let entities = Arc::clone(entities);
        let camera = camera.clone();
        let stdout = Arc::clone(&stdout);
        let thread = thread::spawn(move || {
            let data: *mut u8;
            {
                let mut buffer = buffer.lock().unwrap();
                data = buffer.data.as_mut().unwrap().as_mut_ptr();
                let mut stdout = stdout.lock().unwrap();
                writeln!(stdout, "Thread {:?} - Buffer size: {}", thread::current().id(), end - start);
            }
            for offset in (start..end).step_by(4) {
                let x = (offset / 4) % image_width;
                let y = (offset / 4) / image_width;

                let mut color = vec3(0.0, 0.0, 0.0);
                for _ in 0..camera.samples_per_pixel {
                    let ray = camera.get_ray(x, y);
                    color += camera.ray_color(&ray, &entities, 0);
                } 
                color *= camera.pixel_samples_scale;
                let intensity = Interval::new(0.0, 0.999);
                let ir = (255.99 * intensity.clamp(linear_to_gamma(color.x))) as u8;
                let ig = (255.99 * intensity.clamp(linear_to_gamma(color.y))) as u8;
                let ib = (255.99 * intensity.clamp(linear_to_gamma(color.z))) as u8;
                unsafe {
                    data.add(offset as usize).write(ib);
                    data.add((offset+1) as usize).write(ig);
                    data.add((offset+2) as usize).write(ir);
                    data.add((offset+3) as usize).write(0xFF);
                }
                let pb = pb_clone.lock().unwrap();
                pb.inc(1);
            }
        });
        threads.push(thread);
    }

    for thread in threads {
        thread.join().unwrap();
    }
}

fn main() {
    let mut window = Window::new("RustRT");
    //let aspect_ratio = window.dim.width as f32 / window.dim.height as f32; //16f32/9f32;
    let image_width = window.dim.width as u32;
    let image_height = window.dim.height as u32;
    assert!(image_height > 1);
    window.buffer = Arc::new(Mutex::new(create_back_buffer(image_width as i32, image_height as i32)));
    let camera = Camera::new(image_width, image_height, 20.0, &vec3(13.0, 2.0, 3.0), &vec3(0.0, 0.0, 0.0));

    let mut entities = EntityList::new();

    let material_ground: Arc<dyn Material> = Arc::new(Lambertian{
        albedo: vec3(0.5, 0.5, 0.5),
    });
    let material_1 : Arc<dyn Material> = Arc::new(Dielectric{
        refraction_index: 1.5
    });
    let material_2 : Arc<dyn Material> = Arc::new(Lambertian {
        albedo: vec3(0.4, 0.2, 0.1)
    });
    let material_3 : Arc<dyn Material> = Arc::new(Metal{
        albedo: vec3(0.7, 0.6, 0.5),
        fuzz: 0.0
    });

    let center = vec3(0.0, -1000.0, -1.0);
    let radius = 1000.0;
    entities.list.push(Box::new(Sphere::new(center, radius, Arc::clone(&material_ground))));
    
    for a in -11..11 {
        for b in -11..11 {
            let choose_mat = rand_f32();
            let center = vec3((a as f32)+0.9*rand_f32(), 0.2, (b as f32)*0.9*rand_f32());
            if (center - vec3(4.0, 0.2, 0.0)).length() > 0.9 {
                if choose_mat < 0.8 {
                    let albedo = Vec3::random() * Vec3::random();
                    let material : Arc<dyn Material> = Arc::new(Lambertian{
                        albedo
                    });
                    entities.list.push(Box::new(Sphere::new(center, 0.2, material)));
                } else if choose_mat < 0.95 {
                    let albedo = Vec3::random_range(0.5, 1.0);
                    let fuzz = rand_f32_range(0.0, 0.5);
                    let material : Arc<dyn Material> = Arc::new(Metal{
                        albedo,
                        fuzz
                    });
                    entities.list.push(Box::new(Sphere::new(center, 0.2, material)));
                } else {
                    let material : Arc<dyn Material> = Arc::new(Dielectric{
                        refraction_index: 1.5
                    });
                    entities.list.push(Box::new(Sphere::new(center, 0.2, material)));
                }
            }
        }
    }

    let center = vec3(0.0, 1.0, 0.0);
    let radius = 1.0;
    entities.list.push(Box::new(Sphere::new(center, radius, Arc::clone(&material_1))));
    
    let center = vec3(-4.0, 1.0, 0.0);
    let radius = 0.5;
    entities.list.push(Box::new(Sphere::new(center, radius, Arc::clone(&material_2))));
    
    let center = vec3(4.0, 1.0, 0.0);
    let radius = 0.4;
    entities.list.push(Box::new(Sphere::new(center, radius, Arc::clone(&material_3))));

    let entities = Arc::from(entities);
    let mut first = true;
    loop {
        window.process_messages();
        window.display();
        
        if first {
            render(&window, image_width, image_height, &entities, &camera);
            first = false;
        }
        
        if window.shutdown_requested {
            break;
        }
    }
}
