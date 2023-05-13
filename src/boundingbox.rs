use stl_io::Vector;
pub struct BoundingBox {
    pub x: (f32, f32),
    pub y: (f32, f32),
    pub z: (f32, f32),
}

impl BoundingBox {
    pub fn new() -> Self {
        BoundingBox {
            x: (f32::INFINITY, f32::NEG_INFINITY),
            y: (f32::INFINITY, f32::NEG_INFINITY),
            z: (f32::INFINITY, f32::NEG_INFINITY),
        }
    }
    pub fn update(&mut self, vertex: &Vector<f32>) {
        if vertex[0] > self.x.1 {
            self.x.1 = vertex[0];
        }
        if vertex[0] < self.x.0 {
            self.x.0 = vertex[0];
        }
        if vertex[1] > self.y.1 {
            self.y.1 = vertex[1];
        }
        if vertex[1] < self.y.0 {
            self.y.0 = vertex[1];
        }
        if vertex[2] > self.z.1 {
            self.z.1 = vertex[2];
        }
        if vertex[2] < self.z.0 {
            self.z.0 = vertex[2];
        }
    }
}