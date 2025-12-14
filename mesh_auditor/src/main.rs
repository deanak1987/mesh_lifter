use anyhow::Result;
use std::env;
use std::fs::File;
use std::io::Write;
use marching_cubes::{marching_cubes, Field};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: cargo run -- <scan_file.obj>");
        return Ok(());
    }
    let filename = &args[1];

    println!("-----------------------------------------");
    println!("🧬 VOXEL REMESHER: initializing...");
    println!("-----------------------------------------");

    // 1. Load the messy scan
    let load_options = tobj::LoadOptions { triangulate: true, ..Default::default() };
    let (models, _) = tobj::load_obj(filename, &load_options)?;
    let mesh = &models[0].mesh;

    println!("   • Input Vertices: {}", mesh.positions.len() / 3);

    // 2. Define the resolution (Higher = more detail, slower)
    // For a demo, 50 is fast. For production, you'd want 100-200.
    let resolution = 50; 
    
    // 3. Find the Bounding Box of the object
    let (min_bound, max_bound) = get_bounds(&mesh.positions);
    println!("   • Bounding Box found. Grid size: {}x{}x{}", resolution, resolution, resolution);

    // 4. Create the "Field" (The Voxel Grid)
    // We are creating a "Metaball" effect: The points of the scan emit a 'field'.
    // Where the field is strong, we draw the skin.
    let field = MeshDistanceField {
        positions: &mesh.positions,
        min: min_bound,
        max: max_bound,
        resolution,
    };

    println!("   • Running Marching Cubes (This acts as the 'Shrink Wrap')...");
    
    // 5. Generate the new mesh
    // The '0.5' is the density threshold. 
    let new_mesh = marching_cubes(&field, 0.5);

    println!("   ✅ RE-SKINNING COMPLETE.");
    println!("   • New Vertices: {}", new_mesh.len() / 3);

    // 6. Save the Result
    save_triangles_as_stl(&new_mesh, "repaired_voxel_skin.stl")?;

    Ok(())
}

// --- HELPER STRUCTURES ---

// This struct defines our "Voxel Grid"
struct MeshDistanceField<'a> {
    positions: &'a [f32],
    min: (f32, f32, f32),
    max: (f32, f32, f32),
    resolution: usize,
}

// This implements the trait required by the crate.
// It answers the question: "What is the density at coordinates (x,y,z)?"
impl<'a> Field for MeshDistanceField<'a> {
    fn dimensions(&self) -> [usize; 3] {
        [self.resolution, self.resolution, self.resolution]
    }

    // This is the heavy lifting.
    // For every voxel, we calculate its value based on proximity to the scan points.
    fn z(&self, x: usize, y: usize, z: usize) -> f64 {
        // Convert grid coordinates (0, 1, 2) to World Coordinates (0.5mm, 1.0mm...)
        let step_x = (self.max.0 - self.min.0) / self.resolution as f32;
        let step_y = (self.max.1 - self.min.1) / self.resolution as f32;
        let step_z = (self.max.2 - self.min.2) / self.resolution as f32;

        let world_x = self.min.0 + (x as f32 * step_x);
        let world_y = self.min.1 + (y as f32 * step_y);
        let world_z = self.min.2 + (z as f32 * step_z);

        // SIMPLE ALGORITHM (Metaball Style):
        // Find the distance to the CLOSEST vertex in the original scan.
        // In a real production app, you would use a 'KdTree' to make this instant.
        // Here, we loop through points (Slow but simple for code clarity).
        
        let mut min_dist_sq = f32::MAX;
        
        // OPTIMIZATION: Just check every 10th point to speed up the demo
        for i in (0..self.positions.len()).step_by(30) {
            let px = self.positions[i];
            let py = self.positions[i+1];
            let pz = self.positions[i+2];

            let dist_sq = (px - world_x).powi(2) + (py - world_y).powi(2) + (pz - world_z).powi(2);
            if dist_sq < min_dist_sq {
                min_dist_sq = dist_sq;
            }
        }

        // Return a density value. 
        // If we are close to a point, return 1.0. If far, return 0.0.
        // We use an inverse distance function.
        let threshold = (step_x * 3.0).powi(2); // Radius of influence
        if min_dist_sq < threshold {
            return 1.0;
        }
        0.0
    }
}

// Helper to find the size of the object
fn get_bounds(positions: &[f32]) -> ((f32, f32, f32), (f32, f32, f32)) {
    let mut min = (f32::MAX, f32::MAX, f32::MAX);
    let mut max = (f32::MIN, f32::MIN, f32::MIN);

    for chunk in positions.chunks(3) {
        if let [x, y, z] = chunk {
            if *x < min.0 { min.0 = *x; }
            if *y < min.1 { min.1 = *y; }
            if *z < min.2 { min.2 = *z; }
            
            if *x > max.0 { max.0 = *x; }
            if *y > max.1 { max.1 = *y; }
            if *z > max.2 { max.2 = *z; }
        }
    }
    // Add some padding so the object isn't touching the edge of the grid
    let padding = 0.2;
    (
        (min.0 - padding, min.1 - padding, min.2 - padding),
        (max.0 + padding, max.1 + padding, max.2 + padding)
    )
}

// Basic STL Writer for the output
fn save_triangles_as_stl(triangles: &[usize], filename: &str) -> Result<()> {
    // The 'marching_cubes' crate returns a flat list of coordinates
    // [x1, y1, z1, x2, y2, z2, ...]
    
    let mut file = File::create(filename)?;
    writeln!(file, "solid voxel_skin")?;

    for chunk in triangles.chunks(9) {
        // chunk contains 3 vertices (9 floats)
        writeln!(file, "facet normal 0 0 0")?;
        writeln!(file, "outer loop")?;
        writeln!(file, "vertex {} {} {}", chunk[0], chunk[1], chunk[2])?; // V1
        writeln!(file, "vertex {} {} {}", chunk[3], chunk[4], chunk[5])?; // V2
        writeln!(file, "vertex {} {} {}", chunk[6], chunk[7], chunk[8])?; // V3
        writeln!(file, "endloop")?;
        writeln!(file, "endfacet")?;
    }
    
    writeln!(file, "endsolid voxel_skin")?;
    Ok(())
}
