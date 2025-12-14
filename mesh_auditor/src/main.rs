use anyhow::Result;
use std::env;

fn main() -> Result<()> {
    // 1. Get the filename from the command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: cargo run -- <path_to_model.obj>");
        return Ok(());
    }
    let filename = &args[1];

    println!("-----------------------------------------");
    println!("🔍 STARTING AUDIT: {}", filename);
    println!("-----------------------------------------");

    // 2. Load the OBJ file
    let load_options = tobj::LoadOptions {
        triangulate: true,
        ..Default::default()
    };

    let (models, _materials) = tobj::load_obj(filename, &load_options)?;

    // 3. Analyze the Geometry
    let mut total_vertices = 0;
    let mut total_faces = 0;

    for (i, m) in models.iter().enumerate() {
        let mesh = &m.mesh;
        let vertex_count = mesh.positions.len() / 3; // X, Y, Z coordinates
        let face_count = mesh.indices.len() / 3;     // 3 indices per triangle

        println!("\nObject #{}: {}", i + 1, m.name);
        println!("   • Vertices: {}", vertex_count);
        println!("   • Faces (Triangles): {}", face_count);
                                                                                                                                                        
        // CHECK FOR "HEAVINESS"
        if face_count > 100_000 {
            println!("   ⚠️  WARNING: High Polygon Count! Candidate for decimation.");
        } else {
            println!("   ✅ Status: Web Safe");
        }
        total_vertices += vertex_count;
        total_faces += face_count;
    }

    println!("\n-----------------------------------------");
    println!("📊 FINAL REPORT");
    println!("   Total Objects: {}", models.len());
    println!("   Total Vertices: {}", total_vertices);
    println!("   Total Triangles: {}", total_faces);
    println!("-----------------------------------------");

    Ok(())
}
                                                                                                                                                                                                                                                                