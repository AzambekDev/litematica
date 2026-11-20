//! Rendering utilities: Color4f, 3D wireframe boxes, shaded overlays, and ghost blocks.

use crate::malilib::math::{BlockBox, BlockPos, Vec3d};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color4f {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color4f {
    pub const WHITE: Color4f = Color4f { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
    pub const BLACK: Color4f = Color4f { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const RED: Color4f = Color4f { r: 1.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const GREEN: Color4f = Color4f { r: 0.0, g: 1.0, b: 0.0, a: 1.0 };
    pub const BLUE: Color4f = Color4f { r: 0.0, g: 0.5, b: 1.0, a: 1.0 };
    pub const CYAN: Color4f = Color4f { r: 0.0, g: 1.0, b: 1.0, a: 1.0 };
    pub const MAGENTA: Color4f = Color4f { r: 1.0, g: 0.0, b: 1.0, a: 1.0 };
    pub const ORANGE: Color4f = Color4f { r: 1.0, g: 0.5, b: 0.0, a: 1.0 };

    // Standard Litematica mismatch overlay colors
    pub const OVERLAY_MISSING: Color4f = Color4f { r: 0.2, g: 0.6, b: 1.0, a: 0.35 }; // Translucent cyan/blue
    pub const OVERLAY_WRONG_BLOCK: Color4f = Color4f { r: 1.0, g: 0.1, b: 0.1, a: 0.5 }; // Bright red
    pub const OVERLAY_WRONG_STATE: Color4f = Color4f { r: 1.0, g: 0.6, b: 0.0, a: 0.5 }; // Orange
    pub const OVERLAY_EXTRA_BLOCK: Color4f = Color4f { r: 0.9, g: 0.1, b: 0.9, a: 0.5 }; // Magenta

    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn from_hex(hex: u32) -> Self {
        let a = ((hex >> 24) & 0xFF) as f32 / 255.0;
        let r = ((hex >> 16) & 0xFF) as f32 / 255.0;
        let g = ((hex >> 8) & 0xFF) as f32 / 255.0;
        let b = (hex & 0xFF) as f32 / 255.0;
        Self { r, g, b, a: if a == 0.0 { 1.0 } else { a } }
    }

    pub fn to_hex(&self) -> u32 {
        let a = ((self.a * 255.0).clamp(0.0, 255.0) as u32) << 24;
        let r = ((self.r * 255.0).clamp(0.0, 255.0) as u32) << 16;
        let g = ((self.g * 255.0).clamp(0.0, 255.0) as u32) << 8;
        let b = (self.b * 255.0).clamp(0.0, 255.0) as u32;
        a | r | g | b
    }
}

#[derive(Debug, Clone)]
pub struct LinePrimitive {
    pub start: Vec3d,
    pub end: Vec3d,
    pub color: Color4f,
    pub width: f32,
}

#[derive(Debug, Clone)]
pub struct BoxFacePrimitive {
    pub vertices: [Vec3d; 4],
    pub color: Color4f,
}

pub struct RenderBatch {
    pub lines: Vec<LinePrimitive>,
    pub faces: Vec<BoxFacePrimitive>,
}

impl RenderBatch {
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            faces: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.lines.clear();
        self.faces.clear();
    }

    pub fn add_box_wireframe(&mut self, bbox: &BlockBox, color: Color4f, line_width: f32) {
        let min = Vec3d::new(bbox.min_x as f64, bbox.min_y as f64, bbox.min_z as f64);
        let max = Vec3d::new((bbox.max_x + 1) as f64, (bbox.max_y + 1) as f64, (bbox.max_z + 1) as f64);

        // 12 edges of a cuboid
        let corners = [
            min,
            Vec3d::new(max.x, min.y, min.z),
            Vec3d::new(max.x, min.y, max.z),
            Vec3d::new(min.x, min.y, max.z),
            Vec3d::new(min.x, max.y, min.z),
            Vec3d::new(max.x, max.y, min.z),
            max,
            Vec3d::new(min.x, max.y, max.z),
        ];

        let edges = [
            (0, 1), (1, 2), (2, 3), (3, 0), // bottom face
            (4, 5), (5, 6), (6, 7), (7, 4), // top face
            (0, 4), (1, 5), (2, 6), (3, 7), // vertical pillars
        ];

        for &(i, j) in &edges {
            self.lines.push(LinePrimitive {
                start: corners[i],
                end: corners[j],
                color,
                width: line_width,
            });
        }
    }

    pub fn add_shaded_block(&mut self, pos: &BlockPos, color: Color4f) {
        let min_x = pos.x as f64;
        let min_y = pos.y as f64;
        let min_z = pos.z as f64;
        let max_x = min_x + 1.0;
        let max_y = min_y + 1.0;
        let max_z = min_z + 1.0;

        // Bottom face (y = min_y)
        self.faces.push(BoxFacePrimitive {
            vertices: [
                Vec3d::new(min_x, min_y, min_z),
                Vec3d::new(max_x, min_y, min_z),
                Vec3d::new(max_x, min_y, max_z),
                Vec3d::new(min_x, min_y, max_z),
            ],
            color,
        });

        // Top face (y = max_y)
        self.faces.push(BoxFacePrimitive {
            vertices: [
                Vec3d::new(min_x, max_y, min_z),
                Vec3d::new(min_x, max_y, max_z),
                Vec3d::new(max_x, max_y, max_z),
                Vec3d::new(max_x, max_y, min_z),
            ],
            color,
        });

        // North face (z = min_z)
        self.faces.push(BoxFacePrimitive {
            vertices: [
                Vec3d::new(min_x, min_y, min_z),
                Vec3d::new(min_x, max_y, min_z),
                Vec3d::new(max_x, max_y, min_z),
                Vec3d::new(max_x, min_y, min_z),
            ],
            color,
        });

        // South face (z = max_z)
        self.faces.push(BoxFacePrimitive {
            vertices: [
                Vec3d::new(min_x, min_y, max_z),
                Vec3d::new(max_x, min_y, max_z),
                Vec3d::new(max_x, max_y, max_z),
                Vec3d::new(min_x, max_y, max_z),
            ],
            color,
        });

        // West face (x = min_x)
        self.faces.push(BoxFacePrimitive {
            vertices: [
                Vec3d::new(min_x, min_y, min_z),
                Vec3d::new(min_x, min_y, max_z),
                Vec3d::new(min_x, max_y, max_z),
                Vec3d::new(min_x, max_y, min_z),
            ],
            color,
        });

        // East face (x = max_x)
        self.faces.push(BoxFacePrimitive {
            vertices: [
                Vec3d::new(max_x, min_y, min_z),
                Vec3d::new(max_x, max_y, min_z),
                Vec3d::new(max_x, max_y, max_z),
                Vec3d::new(max_x, min_y, max_z),
            ],
            color,
        });
    }
}
