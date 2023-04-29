use pyo3::prelude::*;
use std::fs::OpenOptions;

/// Formats the sum of two numbers as string.
#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

#[pyfunction]
fn read_stl(fname: String) -> PyResult<()> {
    let mut file = OpenOptions::new().read(true).open(fname).unwrap();
    let stl = stl_io::read_stl(&mut file).unwrap();
    let size_hint = stl_io::create_stl_reader(&mut file).unwrap().size_hint();
    // print size_hint to stdout
    println!("{:?}", size_hint);
    // loop over all enumerated faces

    // build up events structure
    let mut events: Vec<(f32, String, usize)> = Vec::new();
    for (i, face) in stl.faces.iter().enumerate() {
        let mut empty_vector: Vec<f32> = Vec::new();
        for vertex in face.vertices {
            let actual_vertex = stl.vertices[vertex];
            empty_vector.push(actual_vertex[2]);
        }
        let min_of_empy_vector = empty_vector.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        let max_of_empy_vector = empty_vector.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        // print min and max
        // push a tuple of z coord, 'start' or 'end' and face index
        events.push((*min_of_empy_vector, "start".to_string(), i));
        events.push((*max_of_empy_vector, "end".to_string(), i));
    }
    // print events to stdout
    println!("{:?}", events);
    // looop over events
    for event in events {
        // if start, push face index to stack
        // if end, pop face index from stack
        // if stack is empty, push face index to array
        // if stack is not empty, push face index to array
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