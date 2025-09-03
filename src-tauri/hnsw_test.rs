use hnsw::dist::Cosine;
use hnsw::Hnsw;

fn main() {
    // Test the correct constructor signature
    let mut hnsw: Hnsw<f32, Cosine> = Hnsw::default();
    println!("HNSW created successfully with default constructor!");

    // Test adding a point
    let vector = vec![1.0, 0.0, 0.0];
    hnsw.add_point(&vector, 0);
    println!("Point added successfully!");
}
