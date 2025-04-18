use crate::math::mat3::dot_v3;
use crate::math::rand::rand_f32;
use crate::math::{
    mat3::Mat3,
    vec2::Vec2,
    vec3::{dot, reflect, Vec3},
};
use crate::texture::TextureSampler;
use crate::{entities::entity::HitRecord, Ray};
use core::f32;
use std::any::Any;
use std::fmt::Debug;
pub trait Material: Debug + Any + Sync + Send {
    fn scatter_pdf(&self, _ray_in: &Ray, _record: &HitRecord, _ray_scattered: &Ray) -> f32 {
        0.0
    }

    fn scatter(
        &self,
        _ray: &Ray,
        _hit_record: &HitRecord,
        _attenuation: &mut Vec3,
        _scattered: &mut Ray,
        _pdf: &mut f32,
    ) -> bool {
        false
    }

    fn emitted(&self, _ray_in: &Ray, _record: &HitRecord, _uv: &Vec2, _position: &Vec3) -> Vec3 {
        Vec3::zero()
    }
    fn as_any(&self) -> &dyn Any;
}

impl PartialEq for dyn Material {
    fn eq(&self, other: &Self) -> bool {
        self.as_any().type_id() == other.as_any().type_id()
    }
}

#[derive(Debug)]
pub struct Lambertian {
    pub albedo: Box<dyn TextureSampler>,
}

#[derive(Debug)]
pub struct Metal {
    pub albedo: Vec3,
    pub fuzz: f32,
}

#[derive(Debug)]
pub struct Dielectric {
    pub refraction_index: f32,
}

impl Dielectric {
    pub fn refract(uv: &Vec3, normal: &Vec3, etai_over_etat: f32) -> Vec3 {
        let cos_theta = f32::min(dot(&-uv, normal), 1.0);
        let r_out_perp = etai_over_etat * (*uv + (cos_theta * normal));
        let r_out_parallel = -f32::sqrt(f32::abs(1.0 - r_out_perp.length_squared())) * *normal;
        r_out_perp + r_out_parallel
    }

    pub fn reflectance(cosine: f32, ri: f32) -> f32 {
        let mut r0 = (1.0 - ri) / (1.0 + ri);
        r0 = r0 * r0;
        r0 + (1.0 - r0) * f32::powi(1.0 - cosine, 5)
    }
}

#[derive(Debug)]
pub struct DiffuseLight {
    pub emit: Box<dyn TextureSampler>,
}

#[derive(Debug)]
pub struct Isotropic {
    pub albedo: Box<dyn TextureSampler>,
}

impl Material for Lambertian {
    fn scatter_pdf(&self, _ray_in: &Ray, _record: &HitRecord, _ray_scattered: &Ray) -> f32 {
        /*
        let cos_theta = dot(&record.normal, &ray_scattered.direction);
        if cos_theta < 0.0 {
            0.0
        } else {
            cos_theta / f32::consts::PI
        }
         */
        1.0 / (2.0 * std::f32::consts::PI)
    }

    fn scatter(
        &self,
        _ray: &Ray,
        hit_record: &HitRecord,
        attenuation: &mut Vec3,
        scattered: &mut Ray,
        pdf: &mut f32,
    ) -> bool {
        let uwv = Mat3::get_orthonormal_basis(&hit_record.normal);
        let scatter_direction = dot_v3(
            &uwv.transpose(),
            &Vec3::random_cosine_hemisphere_direction(),
        );
        *scattered = Ray::new(hit_record.position, scatter_direction.normalize());
        *attenuation = self
            .albedo
            .as_ref()
            .value(&hit_record.uv, &hit_record.position);
        *pdf = dot(&uwv[2], &scattered.direction) / f32::consts::PI;
        true
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Material for Metal {
    fn scatter(
        &self,
        ray: &Ray,
        hit_record: &HitRecord,
        attenuation: &mut Vec3,
        scattered: &mut Ray,
        _pdf: &mut f32,
    ) -> bool {
        let reflected = reflect(&ray.direction, &hit_record.normal).normalize()
            + (self.fuzz * Vec3::random_unit());
        *scattered = Ray::new(hit_record.position, reflected);
        *attenuation = self.albedo;
        dot(&scattered.direction, &hit_record.normal) > 0.0
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Material for Dielectric {
    fn scatter(
        &self,
        ray: &Ray,
        hit_record: &HitRecord,
        attenuation: &mut Vec3,
        scattered: &mut Ray,
        _pdf: &mut f32,
    ) -> bool {
        *attenuation = Vec3::one();
        let ri = if hit_record.front_face {
            1.0 / self.refraction_index
        } else {
            self.refraction_index
        };
        let direction = ray.direction.normalize();
        let cos_theta = f32::min(dot(&-direction, &hit_record.normal), 1.0);
        let sin_theta = f32::sqrt(1.0 - cos_theta * cos_theta);
        let cannot_refract = (ri * sin_theta) > 1.0;
        let ref_direction =
            if cannot_refract || (Dielectric::reflectance(cos_theta, ri) > rand_f32()) {
                reflect(&direction, &hit_record.normal)
            } else {
                Dielectric::refract(&direction, &hit_record.normal, ri)
            };

        *scattered = Ray::new(hit_record.position, ref_direction);
        true
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Material for DiffuseLight {
    fn emitted(&self, _ray_in: &Ray, record: &HitRecord, uv: &Vec2, position: &Vec3) -> Vec3 {
        if !record.front_face {
            return Vec3::zero();
        }
        self.emit.as_ref().value(uv, position)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Material for Isotropic {
    fn scatter_pdf(&self, _ray_in: &Ray, _record: &HitRecord, _ray_scattered: &Ray) -> f32 {
        1.0 / (4.0 * f32::consts::PI)
    }

    fn scatter(
        &self,
        _ray: &Ray,
        hit_record: &HitRecord,
        attenuation: &mut Vec3,
        scattered: &mut Ray,
        pdf: &mut f32,
    ) -> bool {
        *scattered = Ray::new(hit_record.position, Vec3::random_unit());
        *attenuation = self
            .albedo
            .as_ref()
            .value(&hit_record.uv, &hit_record.position);
        *pdf = 1.0 / (4.0 * f32::consts::PI);
        true
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
