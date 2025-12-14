use anyhow::Result;
use std::env;
use std::fs::File;
use std::io::Write; // Needed to write to files

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: cargo run -- <path_to_model.obj>");
        return Ok(());
    }
    let filename = &args[1];

    // 1. Load the OBJ
    println!("📖 Loading {}...", filename);
    let load_options = tobj::LoadOptions {
        triangulate: true,
        ..Default::default()
    };
    let (models, _materials) = tobj::load_obj(filename, &load_options)?;

    // 2. Process the first object found
    if let Some(m) = models.first() {
        let mesh = &m.mesh;
        println!("✅ Model Loaded. Vertices: {}", mesh.positions.len() / 3);

        // 3. Define the output filename
        let output_filename = "output.stl";
        
        // 4. Call our new 'Writer' function
        save_as_stl(mesh, output_filename)?;
        println!("💾 SUCCESS! Saved converted file to: {}", output_filename);
    } else {
        println!("❌ No 3D objects found in file.");
    }

    Ok(())
}

// THE NEW HEAVY LIFTING FUNCTION
// This takes raw memory data and formats it into the STL standard
fn save_as_stl(mesh: &tobj::Mesh, filename: &str) -> Result<()> {
    let mut file = File::create(filename)?;

    // Write the STL Header (Standard format name)
    writeln!(file, "solid rust_converted_mesh")?;

    // Loop through every triangle (face)
    // The indices list tells us which vertices make up a triangle
    // e.g., Triangle 1 uses Vertex 0, 1, and 2
    for chunk in mesh.indices.chunks(3) {
        if let [i1, i2, i3] = chunk {
            // Get the X, Y, Z for each of the 3 vertices
            let v1 = get_vertex(mesh, *i1 as usize);
            let v2 = get_vertex(mesh, *i2 as usize);
            let v3 = get_vertex(mesh, *i3 as usize);

            // Write the "facet" (triangle) to the file
            // Note: We are using a dummy Normal (0 0 0) for simplicity
            writeln!(file, "facet normal 0.0 0.0 0.0")?;
            writeln!(file, "    outer loop")?;
            writeln!(file, "        vertex {} {} {}", v1.0, v1.1, v1.2)?;
            writeln!(file, "        vertex {} {} {}", v2.0, v2.1, v2.2)?;
            writeln!(file, "        vertex {} {} {}", v3.0, v3.1, v3.2)?;
            writeln!(file, "    endloop")?;
            writeln!(file, "endfacet")?;
        }
    }

    writeln!(file, "endsolid rust_converted_mesh")?;
    Ok(())
}

// Helper to extract X, Y, Z from the flat list
fn get_vertex(mesh: &tobj::Mesh, index: usize) -> (f64, f64, f64) {
    let x = mesh.positions[index * 3] as f64;
    let y = mesh.positions[index * 3 + 1] as f64;
    let z = mesh.positions[index * 3 + 2] as f64;
    (x, y, z)
}
