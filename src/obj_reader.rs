use crate::math::vec3::{cross, Vec3};
use crate::mesh::{Face, Mesh};
use core::str;
use std::io::Read;

pub fn read_obj(path: &str) -> Mesh {
    let mut file = std::fs::File::open(path).unwrap();
    let mut buffer = Vec::new();
    let size = file.read_to_end(&mut buffer).unwrap();
    assert!(size == buffer.len());
    parse_obj(str::from_utf8(buffer.as_slice()).unwrap())
}

pub fn parse_obj(data: &str) -> Mesh {
    let mut mesh = Mesh::new();
    for line in data.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if let Some(v) = trimmed.strip_prefix("v ") {
            let position: Vec<f32> = v
                .split_whitespace()
                .filter_map(|s| s.parse::<f32>().ok())
                .collect();
            assert!(position.len() == 3);
            mesh.vert_position
                .push(Vec3::new(position[0], position[1], position[2]));
        } else if let Some(vn) = trimmed.strip_prefix("vn ") {
            let normal: Vec<f32> = vn
                .split_whitespace()
                .filter_map(|s| s.parse::<f32>().ok())
                .collect();
            assert!(normal.len() == 3);
            mesh.vert_normal
                .push(Vec3::new(normal[0], normal[1], normal[2]).normalize());
        } else if let Some(f) = trimmed.strip_prefix("f ") {
            let mut pos_idx = [0; 3];
            let mut normal_idx = [0; 3];
            for (i, part) in f.split_whitespace().enumerate() {
                let mut indicies = part.split('/');
                let v_pos_idx = indicies
                    .next()
                    .and_then(|s| s.parse::<usize>().ok())
                    .map(|i| i - 1);
                let _v_tex_idx = indicies
                    .next()
                    .and_then(|s| s.parse::<usize>().ok())
                    .map(|i| i - 1);
                let v_normal_idx = indicies
                    .next()
                    .and_then(|s| s.parse::<usize>().ok())
                    .map(|i| i - 1);
                if let Some(vp) = v_pos_idx {
                    pos_idx[i] = vp;
                }
                if let Some(vn) = v_normal_idx {
                    normal_idx[i] = vn;
                }
            }
            mesh.faces.push(Face {
                vert_pos_idx: pos_idx,
                vert_normal_idx: normal_idx,
            });
        }
    }

    if mesh.vert_normal.is_empty() {
        compute_normals(&mut mesh, true);
    }
    mesh
}

fn compute_normals(mesh: &mut Mesh, smooth: bool) {
    let positions = &mesh.vert_position;
    let mut normals = vec![Vec3::zero(); mesh.faces.len()];
    for (i, face) in mesh.faces.iter_mut().enumerate() {
        let e1 = positions[face.vert_pos_idx[1]] - positions[face.vert_pos_idx[0]];
        let e2 = positions[face.vert_pos_idx[2]] - positions[face.vert_pos_idx[0]];
        let normal = cross(&e1, &e2).normalize();

        normals[i] = normal;
        face.vert_normal_idx = [i, i, i];
    }
    mesh.vert_normal = normals;

    if smooth {
        let mut normal_acc = vec![Vec3::zero(); mesh.vert_position.len()];
        let mut count = vec![0; mesh.vert_position.len()];
        for (i, face) in mesh.faces.iter_mut().enumerate() {
            let normal = mesh.vert_normal[i];
            for (j, &v_idx) in face.vert_pos_idx.iter().enumerate() {
                normal_acc[v_idx] += normal;
                face.vert_normal_idx[j] = v_idx;
                count[v_idx] += 1;
            }
        }

        for (i, normal) in normal_acc.iter_mut().enumerate() {
            *normal = (*normal / count[i] as f32).normalize();
        }
        mesh.vert_normal = normal_acc;
    }
}
