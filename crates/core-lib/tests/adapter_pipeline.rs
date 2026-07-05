use augmented_gaussian_core::config::{ReconstructionMethod, VoxelBackend};
use augmented_gaussian_core::{ProcessConfig, RecipeBundle, process_file};
use std::fs;

#[test]
fn external_adapter_pipeline_writes_occlusion_glb() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.splat");
    fs::write(&input, splat_bytes()).unwrap();
    let out = dir.path().join("out");
    let mut config = ProcessConfig::default();
    config.reconstruction.method = ReconstructionMethod::Poisson;
    config.reconstruction.adapter_command = Some(quoted_adapter_command(false));
    config.reconstruction.timeout_seconds = 5;
    config.voxel.backend = VoxelBackend::Cpu;
    config.navmesh.enabled = false;
    config.export.write_webar_zip = false;

    let output = process_file(&input, &out, &config, &RecipeBundle::default()).unwrap();

    assert!(out.join("occlusion.glb").exists());
    assert_eq!(output.manifest.metrics.reconstruction_method, "poisson");
    assert_eq!(
        output
            .manifest
            .metrics
            .reconstruction_adapter_status
            .as_deref(),
        Some("ok")
    );
    assert_eq!(output.manifest.metrics.voxel_solid_cells, 0);
    assert!(
        output
            .manifest
            .metrics
            .collision_warnings
            .iter()
            .any(|warning| warning == "fake adapter warning")
    );
}

#[test]
fn external_adapter_mesh_outside_out_dir_is_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.splat");
    fs::write(&input, splat_bytes()).unwrap();
    let out = dir.path().join("out");
    let mut config = ProcessConfig::default();
    config.reconstruction.method = ReconstructionMethod::Sugar;
    config.reconstruction.adapter_command = Some(quoted_adapter_command(true));
    config.reconstruction.timeout_seconds = 5;
    config.navmesh.enabled = false;
    config.export.write_webar_zip = false;

    let err = process_file(&input, &out, &config, &RecipeBundle::default())
        .unwrap_err()
        .to_string();

    assert!(err.contains("adapter mesh output must stay inside resolved outDir"));
}

fn quoted_adapter_command(outside: bool) -> String {
    let adapter = env!("CARGO_BIN_EXE_fake_mesh_adapter");
    if outside {
        format!("\"{adapter}\" --outside")
    } else {
        format!("\"{adapter}\"")
    }
}

fn splat_bytes() -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&0.0f32.to_le_bytes());
    bytes.extend_from_slice(&0.0f32.to_le_bytes());
    bytes.extend_from_slice(&0.0f32.to_le_bytes());
    bytes.extend_from_slice(&0.3f32.to_le_bytes());
    bytes.extend_from_slice(&0.3f32.to_le_bytes());
    bytes.extend_from_slice(&0.3f32.to_le_bytes());
    bytes.extend_from_slice(&[128, 128, 128, 255, 128, 128, 128, 128]);
    bytes
}
