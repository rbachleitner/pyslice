use image::{ImageBuffer, Rgb};
use pyo3::prelude::*;
use std::collections::{HashMap, HashSet};
use std::fs::OpenOptions;
use stl_io::{IndexedTriangle, Vector};
use std::sync::mpsc::channel;
use threadpool::ThreadPool;
pub mod boundingbox;

fn get_intersecting_points_2(
    face: &IndexedTriangle,
    verts: &HashMap<usize, Vector<f32>>,
    z: &f32,
) -> Vec<(f32, f32, f32)> {
    let mut ret: Vec<(f32, f32, f32)> = Vec::new();
    let vertices = vec![
        verts[&face.vertices[0]],
        verts[&face.vertices[1]],
        verts[&face.vertices[2]],
    ];
    let max_z_index = find_index_of_vector_with_greatest_z_value(vertices, z);
    let index_0 = max_z_index;
    let index_1 = (max_z_index + 1) % 3;
    let index_2 = (max_z_index + 2) % 3;
    let edge_1 = (
        verts[&face.vertices[index_0]],
        verts[&face.vertices[index_1]],
    );
    let edge_2 = (
        verts[&face.vertices[index_1]],
        verts[&face.vertices[index_2]],
    );
    let edge_3 = (
        verts[&face.vertices[index_2]],
        verts[&face.vertices[index_0]],
    );
    let edges = vec![edge_1, edge_2, edge_3];
    // corner case: edge_1.0[2] == edge_1.1[2]
    // corner case: edge_1.0[2] == edge_1.1[2] == edge_2.1[2]
    for edge in edges {
        if edge.0[2] == *z {
            ret.push((edge.0[0], edge.0[1], edge.0[2]));
        } else if (edge.0[2] < *z && edge.1[2] > *z) || (edge.0[2] > *z && edge.1[2] < *z) {
            let denom = edge.1[2] - edge.0[2];
            let k = (z - edge.0[2]) / denom;
            let _x = (edge.1[0] - edge.0[0]) * k + edge.0[0];
            let _y = (edge.1[1] - edge.0[1]) * k + edge.0[1];
            let _calculated_z = (edge.1[2] - edge.0[2]) * k + edge.0[2];
            ret.push((_x, _y, _calculated_z));
        }
    }
    return ret;
}

fn paint_plane(
    z: f32,
    subfaces: Vec<IndexedTriangle>,
    subvertices: HashMap<usize, Vector<f32>>,
    pixels: &mut Vec<Vec<i32>>
) {
    let mut polyline: Vec<Vec<(f32, f32, f32)>> = Vec::new();
    for face in subfaces {
        let intersecting_points = get_intersecting_points_2(&face, &subvertices, &z);
        if intersecting_points.len() == 2 {
            polyline.push(intersecting_points);
        } else if intersecting_points.len() == 3 {
            for i in 0..3 {
                polyline.push(vec![
                    intersecting_points[i],
                    intersecting_points[(i + 1) % 3],
                ]);
            }
        }
    }
    let line_events = generate_line_events(&polyline);
    let mut j = 0;
    let mut x = 0.0;
    let mut current_line_indices: HashSet<usize> = HashSet::new();
    while j < line_events.len() {
        let (line_event_x, line_event_type, line_event_index) = &line_events[j];
        if line_event_x > &x {
            // paint y
            let mut ys: Vec<(f32, i32)> = Vec::new();
            for n in current_line_indices.iter() {
                ys.push(generate_y(polyline[*n][0], polyline[*n][1], x));
            }
            ys.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            let mut yi = 0;
            let mut inside = 0;
            for (target_y, inside_change) in ys {
                // round target_y
                let target_y_rounded = target_y.round() as usize;
                if inside > 0 {
                    for _y_idx in yi..target_y_rounded {
                        pixels[x.round() as usize][_y_idx]= 1;
                    }
                }
                inside += inside_change;
                yi = target_y_rounded;
            }
            assert!(inside == 0);
            x += 1.0;
        } else if line_event_x <= &x && line_event_type == "start" {
            assert!(!current_line_indices.contains(line_event_index));
            current_line_indices.insert(*line_event_index);
            j += 1;
        } else if line_event_x <= &x && line_event_type == "end" {
            assert!(current_line_indices.contains(line_event_index));
            current_line_indices.remove(line_event_index);
            j += 1;
        } else {
            panic!("something went wrong");
        }
    }
    // process line events to paint
    // plane
}

fn find_index_of_vector_with_greatest_z_value(vectors: Vec<Vector<f32>>, z: &f32) -> usize {
    let mut max_z_value = f32::NEG_INFINITY;
    let mut max_z_value_index = 0;
    for (index, vector) in vectors.iter().enumerate() {
        if vector[2] > max_z_value {
            max_z_value = vector[2];
            max_z_value_index = index;
        }
    }
    if vectors[(max_z_value_index + 1) % 3][2] == *z {
        return (max_z_value_index + 1) % 3;
    }
    max_z_value_index
}

fn generate_events(
    stl: &mut stl_io::IndexedMesh,
) -> (Vec<(f32, String, usize)>, boundingbox::BoundingBox) {
    let mut events: Vec<(f32, String, usize)> = Vec::new();
    let mut bb = boundingbox::BoundingBox::new();
    for (i, face) in stl.faces.iter().enumerate() {
        let mut empty_vector: Vec<f32> = Vec::new();
        for vertex in face.vertices {
            let actual_vertex = stl.vertices[vertex];
            empty_vector.push(actual_vertex[2]);
            bb.update(&actual_vertex);
        }
        let min_of_empy_vector = empty_vector
            .iter()
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        let max_of_empy_vector = empty_vector
            .iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        events.push((*min_of_empy_vector, "start".to_string(), i));
        events.push((*max_of_empy_vector, "end".to_string(), i));
    }
    for i in 0..stl.vertices.len() {
        // array of f32
        let new_values = [
            stl.vertices[i][0] - bb.x.0,
            stl.vertices[i][1] - bb.y.0,
            stl.vertices[i][2] - bb.z.0,
        ];
        stl.vertices[i] = Vector::new(new_values);
    }
    for event in events.iter_mut() {
        event.0 -= bb.z.0;
    }
    assert_eq!(events.len(), 2 * stl.faces.len());
    // sort events by z
    events.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    return (events, bb);
}

fn generate_line_events(polyline: &Vec<Vec<(f32, f32, f32)>>) -> Vec<(f32, String, usize)> {
    let mut events: Vec<(f32, String, usize)> = Vec::new();
    for (i, line) in polyline.iter().enumerate() {
        let mut first = line[0];
        let mut second = line[1];
        if first.0 > second.0 {
            //swap
            let tmp = first;
            first = second;
            second = tmp;
        } else if first.0 == second.0 {
            continue;
        }
        events.push((first.0, "start".to_string(), i));
        events.push((second.0, "end".to_string(), i));
    }
    events.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    return events;
}

fn generate_y(p1: (f32, f32, f32), p2: (f32, f32, f32), x: f32) -> (f32, i32) {
    let denom = p2.0 - p1.0;
    let k = (x - p1.0) / denom;
    let y = (p2.1 - p1.1) * k + p1.1;
    let mut inside_change = 0;
    if p1.0 > p2.0 {
        inside_change = -1;
    } else if p1.0 < p2.0 {
        inside_change = 1;
    }
    return (y, inside_change);
}

fn save_img(pixels: &Vec<Vec<i32>>, z: usize, width: usize, height: usize) {
    // create an ImageBuffer from the 2D array
    let img = ImageBuffer::from_fn(width as u32, height as u32, |x, y| {
        let xi = x as usize;
        let yi = y as usize;
        let value = 255 * pixels[xi][yi] as u8;
        Rgb([value, value, value])
    });

    // save the image as a PNG file with height in name
    img.save(format!("{}.png", z)).unwrap();
}

#[pyfunction]
fn read_stl(fname: String, z_step: f32) -> PyResult<()> {
    println!("reading stl...");
    let mut file = OpenOptions::new().read(true).open(fname).unwrap();
    let mut stl = stl_io::read_stl(&mut file).unwrap();
    println!("generating events...");
    let (events, bb) = generate_events(&mut stl);
    // ceiling of mx, my, mz
    let mxi = (bb.x.1 - bb.x.0).ceil() as i32;
    let myi = (bb.y.1 - bb.y.0).ceil() as i32;
    let mzi = (bb.z.1 - bb.z.0).ceil() as i32;
    // print bounding box
    println!("bounding box: {} {} {}", mxi, myi, mzi);
    // array of size mxi x myi x mzi
    println!("events length: {}", events.len());
    assert_eq!(events.len(), 2 * stl.faces.len());
    // print first 10 events to stdout
    // looop over events
    let mut z = 0.0;
    let mut i = 0;
    // init empty current face indices set
    let mut current_face_indices: HashSet<usize> = HashSet::new();
    let pool = ThreadPool::new(8);
    // init 3d array of size 100x100x100
    let (tx, rx) = channel();
    let mut jobs: usize = 0;
    while i < events.len() {
        // unpack event
        let (event_z, event_type, face_index) = &events[i];
        if event_z > &z {
            let subfaces: Vec<IndexedTriangle> = current_face_indices
                .iter()
                .map(|i| stl.faces[*i].clone())
                .collect();
            let mut subvertices: HashMap<usize, Vector<f32>> = HashMap::new();
            for face in subfaces.iter() {
                for vertex in face.vertices.iter() {
                    subvertices.insert(*vertex, stl.vertices[*vertex].clone());
                }
            }
            let tx = tx.clone();
            jobs += 1;
            let mut pixels: Vec<Vec<i32>> =
                vec![vec![0; myi as usize]; mxi as usize];
            pool.execute(move|| {
                paint_plane(z, subfaces, subvertices, &mut pixels);
                save_img(&pixels, z.round() as usize, mxi as usize, myi as usize);
                tx.send(1).expect("channel will be waiting for the pool.")
            });
            println!("saved image {}.png, {}, {}", z.round(), mxi, myi);
            z += z_step;
        } else if event_z <= &z && event_type == "start" {
            // add face index to current face indices set
            // and increment i
            current_face_indices.insert(*face_index);
            i += 1;
        } else if event_z <= &z && event_type == "end" {
            // remove face index from current face indices set
            // and increment i
            current_face_indices.remove(face_index);
            i += 1;
        } else {
            panic!("something went wrong");
        }
    }
    rx.iter().take(jobs).for_each(|_| {});
    Ok(())
}

/// A Python module implemented in Rust.
#[pymodule]
fn pyslice(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(read_stl, m)?)?;
    Ok(())
}
