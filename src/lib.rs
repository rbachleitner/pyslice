use pyo3::prelude::*;
use std::collections::HashSet;
use std::fs::OpenOptions;
use stl_io::Vector;
use image::{ImageBuffer, Rgb};

/// Formats the sum of two numbers as string.
#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
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

fn get_intersecting_points(
    stl: &stl_io::IndexedMesh,
    idx: &usize,
    z: &f32,
) -> Vec<(f32, f32, f32)> {
    let mut ret: Vec<(f32, f32, f32)> = Vec::new();
    let _face = &stl.faces[*idx];
    let vertices = vec![
        stl.vertices[_face.vertices[0]],
        stl.vertices[_face.vertices[1]],
        stl.vertices[_face.vertices[2]],
    ];
    let max_z_index = find_index_of_vector_with_greatest_z_value(vertices, z);
    let index_0 = max_z_index;
    let index_1 = (max_z_index + 1) % 3;
    let index_2 = (max_z_index + 2) % 3;
    let edge_1 = (
        stl.vertices[_face.vertices[index_0]],
        stl.vertices[_face.vertices[index_1]],
    );
    let edge_2 = (
        stl.vertices[_face.vertices[index_1]],
        stl.vertices[_face.vertices[index_2]],
    );
    let edge_3 = (
        stl.vertices[_face.vertices[index_2]],
        stl.vertices[_face.vertices[index_0]],
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

fn generate_events(stl: &stl_io::IndexedMesh) -> (Vec<(f32, String, usize)>, f32, f32, f32) {
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    let mut max_z = f32::NEG_INFINITY;
    let mut events: Vec<(f32, String, usize)> = Vec::new();
    for (i, face) in stl.faces.iter().enumerate() {
        let mut empty_vector: Vec<f32> = Vec::new();
        for vertex in face.vertices {
            let actual_vertex = stl.vertices[vertex];
            empty_vector.push(actual_vertex[2]);
            if actual_vertex[0] > max_x {
                max_x = actual_vertex[0];
            }
            if actual_vertex[1] > max_y {
                max_y = actual_vertex[1];
            }
            if actual_vertex[2] > max_z {
                max_z = actual_vertex[2];
            }
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
    assert_eq!(events.len(), 2 * stl.faces.len());
    // sort events by z
    events.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    return (events, max_x, max_y, max_z);
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


fn save_img(volume: &Vec<Vec<Vec<i32>>>, z: usize, width: usize, height: usize) {
    // create an ImageBuffer from the 2D array
    let img = ImageBuffer::from_fn(width as u32, height as u32, |x, y| {
        let xi = x as usize;
        let yi = y as usize;
        let value = 255*volume[xi][yi][z] as u8;
        Rgb([value, value, value])
    });

    // save the image as a PNG file with height in name
    img.save(format!("{}.png", z)).unwrap();
}


#[pyfunction]
fn read_stl(fname: String, z_step: f32) -> PyResult<()> {
    println!("reading stl...");
    let mut file = OpenOptions::new().read(true).open(fname).unwrap();
    let stl = stl_io::read_stl(&mut file).unwrap();
    println!("generating events...");
    let (events, mx, my, mz) = generate_events(&stl);
    // ceiling of mx, my, mz
    let mxi = mx.ceil() as i32;
    let myi = my.ceil() as i32;
    let mzi = mz.ceil() as i32;
    // print bounding box
    println!("bounding box: {} {} {}", mxi, myi, mzi);
    // array of size mxi x myi x mzi
    let mut volume: Vec<Vec<Vec<i32>>> = vec![vec![vec![0; mzi as usize]; myi as usize]; mxi as usize];
    println!("events length: {}", events.len());
    assert_eq!(events.len(), 2 * stl.faces.len());
    // print first 10 events to stdout
    // looop over events
    let mut z = 0.0;
    let mut i = 0;
    // init empty current face indices set
    let mut current_face_indices: HashSet<usize> = HashSet::new();
    // init 3d array of size 100x100x100
    while i < events.len() {
        // unpack event
        let (event_z, event_type, face_index) = &events[i];
        if event_z > &z {
            // we processed all events for current z
            // print current z, i and length of current face indices set
            println!("z: {}, i: {}, len: {}", z, i, current_face_indices.len());
            // for all faces in the current set
            // assemble polyline
            let mut polyline: Vec<Vec<(f32, f32, f32)>> = Vec::new();
            for idx in &current_face_indices {
                let intersecting_points = get_intersecting_points(&stl, &idx, &z);
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
            // calculate plane by WindingQuery
            // generate line events
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
                        ys.push(generate_y(
                            polyline[*n][0],
                            polyline[*n][1],
                            x,
                        ));
                    }
                    ys.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
                    let mut yi = 0;
                    let mut inside = 0;
                    for (target_y, inside_change) in ys {
                        // round target_y
                        let target_y_rounded = target_y.round() as usize;
                        if inside_change > 0 {
                            for _y_idx in yi..target_y_rounded {
                                volume[x.round() as usize][_y_idx][z.round() as usize] = 1;
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
            save_img(&volume, z.round() as usize, mxi as usize, myi as usize);
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
    Ok(())
}

/// A Python module implemented in Rust.
#[pymodule]
fn pyslice(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    m.add_function(wrap_pyfunction!(read_stl, m)?)?;
    Ok(())
}
