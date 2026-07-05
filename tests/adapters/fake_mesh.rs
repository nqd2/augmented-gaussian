use serde::Deserialize;
use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AdapterRequest {
    out_dir: PathBuf,
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let request: AdapterRequest = serde_json::from_str(&input).unwrap();
    let outside = env::args().any(|arg| arg == "--outside");
    let mesh_path = if outside {
        env::temp_dir().join("fake-mesh-outside.json")
    } else {
        request.out_dir.join("fake-mesh.json")
    };
    let mesh = serde_json::json!({
        "vertices": [
            [-1.0, 0.0, -1.0],
            [1.0, 0.0, -1.0],
            [0.0, 0.0, 1.0]
        ],
        "indices": [0, 1, 2],
        "triangles_before_merge": 1
    });
    fs::write(&mesh_path, serde_json::to_vec_pretty(&mesh).unwrap()).unwrap();
    println!(
        "{}",
        serde_json::json!({
            "meshJson": mesh_path,
            "warnings": ["fake adapter warning"]
        })
    );
}
