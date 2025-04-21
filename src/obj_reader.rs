use crate::math::vec3::Vec3;
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

        if trimmed.starts_with("v ") {
            let position: Vec<f32> = trimmed[2..]
                .split_whitespace()
                .filter_map(|s| s.parse::<f32>().ok())
                .collect();
            assert!(position.len() == 3);
            mesh.vert_position
                .push(Vec3::new(position[0], position[1], position[2]));
        } else if trimmed.starts_with("vn ") {
            let position: Vec<f32> = trimmed[2..]
                .split_whitespace()
                .filter_map(|s| s.parse::<f32>().ok())
                .collect();
            assert!(position.len() == 3);
            mesh.vert_normal
                .push(Vec3::new(position[0], position[1], position[2]));
        } else if trimmed.starts_with("f ") {
            let mut pos_idx = [0; 3];
            let mut normal_idx = [0; 3];
            for (i, part) in trimmed[2..].split_whitespace().enumerate() {
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
                if let (Some(vp), Some(vn)) = (v_pos_idx, v_normal_idx) {
                    pos_idx[i] = vp;
                    normal_idx[i] = vn;
                } else {
                    panic!("corrupted obj file...");
                }
            }
            mesh.faces.push(Face {
                vert_pos_idx: pos_idx,
                vert_normal_idx: normal_idx,
            });
        }
    }
    mesh
}
