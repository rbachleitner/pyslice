use pyo3::prelude::*;
use std::collections::HashSet;
use std::fs::OpenOptions;

/// Formats the sum of two numbers as string.
#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

fn get_intersecting_points(stl: &stl_io::IndexedMesh, idx: &usize, z: &f32) -> Vec<(f32, f32, f32)> {
    let mut ret: Vec<(f32, f32, f32)> = Vec::new();
    let _face = &stl.faces[*idx];
    let edge_1 = (
        stl.vertices[_face.vertices[0]],
        stl.vertices[_face.vertices[1]],
    );
    let edge_2 = (
        stl.vertices[_face.vertices[1]],
        stl.vertices[_face.vertices[2]],
    );
    let edge_3 = (
        stl.vertices[_face.vertices[2]],
        stl.vertices[_face.vertices[0]],
    );
    let edges = vec![edge_1, edge_2, edge_3];
    // corner case: edge_1.0[2] == edge_1.1[2]
    // corner case: edge_1.0[2] == edge_1.1[2] == edge_2.1[2]
    for edge in edges {
        let denom = edge.1[2] - edge.0[2];
        let k = (z - edge.0[2]) / denom;
        let _x = (edge.1[0] - edge.0[0]) * k + edge.0[0];
        let _y = (edge.1[1] - edge.0[1]) * k + edge.0[1];
        let _calculated_z = (edge.1[2] - edge.0[2]) * k + edge.0[2];
        ret.push((_x, _y, _calculated_z));
    }
    return ret;
}

fn generate_events(stl: &stl_io::IndexedMesh) -> Vec<(f32, String, usize)> {
    let mut events: Vec<(f32, String, usize)> = Vec::new();
    for (i, face) in stl.faces.iter().enumerate() {
        let mut empty_vector: Vec<f32> = Vec::new();
        for vertex in face.vertices {
            let actual_vertex = stl.vertices[vertex];
            empty_vector.push(actual_vertex[2]);
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
    return events;
}

#[pyfunction]
fn read_stl(fname: String, z_step: f32) -> PyResult<()> {
    println!("starting read_stl");
    let mut file = OpenOptions::new().read(true).open(fname).unwrap();
    let stl = stl_io::read_stl(&mut file).unwrap();
    let size_hint = stl_io::create_stl_reader(&mut file).unwrap().size_hint();
    // print size_hint to stdout
    println!("{:?}", size_hint);
    // loop over all enumerated faces

    // build up events structure
    let events = generate_events(&stl);
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
                polyline.push(intersecting_points);
            }
            // calculate plane by WindingQuery
            // generate line events
            // process line events to paint
            // plane
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
