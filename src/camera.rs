use crate::entities::entity::*;
use crate::interval::Interval;
use crate::math::rand::{rand_f32, rand_f32_range};
use crate::math::vec3::*;
use crate::pdf::{CosinePDF, PDF};
use crate::ray::Ray;

const UP: Vec3 = Vec3 {
    x: 0.0,
    y: 1.0,
    z: 0.0,
};

#[derive(Debug, Clone, Default)]
pub struct Camera {
    pub camera_position: Vec3,
    pub pixel_delta_x: Vec3,
    pub pixel_delta_y: Vec3,
    pub pixel_origin: Vec3,
    pub samples_per_pixel: u32,
    pub sqrt_spp: u32,
    pub recip_sqrt_spp: f32,
    pub pixel_samples_scale: f32,
    max_ray_bounces: u32,
    defocus_angle: f32,
    defocus_disk_u: Vec3,
    defocus_disk_v: Vec3,
    background_color: Vec3,
}

fn random_disk_vec3() -> Vec3 {
    loop {
        let p = Vec3::new(rand_f32_range(-1.0, 1.0), rand_f32_range(-1.0, 1.0), 0.0);
        if p.length_squared() < 1.0 {
            return p;
        }
    }
}

fn degrees_to_radians(degrees: f32) -> f32 {
    degrees * (std::f32::consts::PI / 180.0)
}

impl Camera {
    pub fn new(
        image_width: u32,
        image_height: u32,
        vfov: f32,
        camera_pos: &Vec3,
        look_at: &Vec3,
    ) -> Self {
        // Camera setup:
        let focus_dist = 10.0f32;
        let camera_position = camera_pos;
        //let focal_length = (look_at - camera_position).length();
        let theta = degrees_to_radians(vfov);
        let h = f32::tan(theta / 2.0);
        let viewport_height = 2.0f32 * h * focus_dist;
        let viewport_width = viewport_height * (image_width as f32 / image_height as f32);

        // compute camera basis vectors
        let w = (camera_pos - look_at).normalize();
        let u = cross(&UP, &w).normalize();
        let v = cross(&w, &u);

        let viewport_x = viewport_width * u;
        let viewport_y = viewport_height * -v;

        let pixel_delta_x = viewport_x / image_width as f32;
        let pixel_delta_y = viewport_y / image_height as f32;
        let viewport_upper_left =
            camera_position - (w * focus_dist) - viewport_x / 2.0 - viewport_y / 2.0;

        let defocus_angle = 0.0;
        let defocus_radius = focus_dist * f32::tan(degrees_to_radians(defocus_angle / 2.0));
        let defocus_disk_u = u * defocus_radius;
        let defocus_disk_v = v * defocus_radius;

        let pixel_origin = viewport_upper_left + 0.5 * (pixel_delta_x + pixel_delta_y);
        let sample_count = 1000;
        Self {
            camera_position: *camera_position,
            pixel_delta_x,
            pixel_delta_y,
            pixel_origin,
            samples_per_pixel: sample_count,
            pixel_samples_scale: 1.0 / sample_count as f32,
            sqrt_spp: (sample_count as f32).sqrt() as u32,
            recip_sqrt_spp: 1.0 / (sample_count as f32).sqrt(),
            max_ray_bounces: 50,
            defocus_angle,
            defocus_disk_u,
            defocus_disk_v,
            background_color: Vec3::new(0.0, 0.0, 0.0),
        }
    }

    pub fn set_background_color(&mut self, color: &Vec3) {
        self.background_color = *color;
    }

    pub fn ray_color(&self, ray: &Ray, entity_list: &EntityList, bounce_idx: u32) -> Vec3 {
        if bounce_idx == self.max_ray_bounces {
            return Vec3::zero();
        }

        let mut record = HitRecord::new();
        if entity_list.hit(ray, &Interval::new(0.001, f32::MAX), &mut record) {
            let mut scattered = Ray::default();
            let mut attenuation = Vec3::zero();
            let Some(material) = record.material.as_ref() else {
                panic!("Material should never be empty")
            };

            let emission_color = record.material.as_ref().unwrap().emitted(
                ray,
                &record,
                &record.uv,
                &record.position,
            );
            let mut pdf_value = 0.0;
            if material.scatter(
                ray,
                &record,
                &mut attenuation,
                &mut scattered,
                &mut pdf_value,
            ) {
                /*
                let on_light = Vec3::new(
                    rand_f32_range(213.0, 343.0),
                    554.0,
                    rand_f32_range(227.0, 332.0),
                );
                let mut to_light = on_light - record.position;
                let dist_sq = to_light.length_squared();
                to_light = to_light.normalize();
                if dot(&to_light, &record.normal) < 0.0 {
                    return emission_color;
                }
                let light_area = (343.0 - 213.0) * (332.0 - 227.0);
                let ligh_cos = f32::abs(to_light.y);
                if ligh_cos < 1e-6 {
                    return emission_color;
                }

                pdf_value = dist_sq / (ligh_cos * light_area);
                let scatter_pdf = material.scatter_pdf(ray, &record, &scattered);
                */
                let surface_pdf = CosinePDF::new(&record.normal);
                scattered = Ray::new(record.position, surface_pdf.generate());
                pdf_value = surface_pdf.value(&scattered.direction);
                let scatter_pdf = material.scatter_pdf(ray, &record, &scattered);
                let bounce_idx = bounce_idx + 1;
                let scatter_color = (attenuation
                    * scatter_pdf
                    * self.ray_color(&scattered, entity_list, bounce_idx))
                    / pdf_value;
                emission_color + scatter_color
            } else {
                emission_color
            }
        } else {
            /*
            let unit_vec = ray.direction.normalize();
            let t = 0.5 * (unit_vec.y + 1.0);
            (1.0 - t) * vec3(1.0, 1.0, 1.0) + t * vec3(0.5, 0.7, 1.0)
            */
            self.background_color
        }
    }

    fn sample_square_stratified(&self, i: u32, j: u32) -> Vec3 {
        Vec3::new(
            ((i as f32 + rand_f32()) * self.recip_sqrt_spp) - 0.5,
            ((j as f32 + rand_f32()) * self.recip_sqrt_spp) - 0.5,
            0.0,
        )
    }

    pub fn defocus_disk_sample(&self) -> Vec3 {
        let p = random_disk_vec3();
        self.camera_position + (p[0] * self.defocus_disk_u) + (p[1] * self.defocus_disk_v)
    }

    pub fn get_ray(&self, x: u32, y: u32, i: u32, j: u32) -> Ray {
        let offset = self.sample_square_stratified(i, j);
        let pixel_pos = self.pixel_origin // pixel _origin is the center of the pixel
            + ((x as f32 + offset.x) * self.pixel_delta_x)
            + ((y as f32 + offset.y) * self.pixel_delta_y);
        let ray_origin = if self.defocus_angle <= 0.0 {
            self.camera_position
        } else {
            self.defocus_disk_sample()
        };
        let ray_direction = pixel_pos - ray_origin;
        Ray::new(ray_origin, ray_direction)
    }
}
