use crate::alignment::{AlignmentTransform, bake_alignment};
use crate::config::{
    EditOperation, ProcessConfig, RecipeBundle, ReconstructionMethod, VoxelBackend,
    VoxelCarveConfig,
};
use crate::error::{AgError, AgResult};
use crate::evaluation::mesh_error_against_splat_centers;
use crate::filters::{
    filter_box, filter_cluster_with_stats, filter_floaters_by_voxel_contribution_with_stats,
    filter_nan, filter_opacity_min, filter_sphere,
};
use crate::glb::{write_mesh_glb, write_navmesh_bin};
use crate::gpu::voxelize_gpu_blocking;
use crate::manifest::{AlignmentManifest, ArtifactManifest, Manifest, Metrics, SourceStats};
use crate::math::{Bounds, Vec3};
use crate::mesh::{Mesh, extract_mesh, validate_mesh};
use crate::navmesh::bake_navmesh;
use crate::readers::{read_source, write_ply, write_sog_bundle};
use crate::voxel::{VoxelParams, carve_grid_with_status, fill_grid_with_status, voxelize_cpu};
use crate::webar::write_webar_zip;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessOutput {
    pub manifest: Manifest,
    pub collision_mesh: Mesh,
    pub navmesh: Option<Mesh>,
}

#[derive(Debug, Clone)]
struct ReconstructionOutcome {
    collision_mesh: Mesh,
    table: crate::SplatTable,
    reconstruction_method: String,
    reconstruction_ms: u128,
    reconstruction_adapter_status: Option<String>,
    gpu_voxel_ms: Option<u128>,
    cpu_gpu_voxel_mismatches: Option<usize>,
    gpu_voxel_speedup: Option<f64>,
    voxel_backend: String,
    voxel_ms: u128,
    cpu_voxel_ms: u128,
    floater_filter_input_count: usize,
    floater_filter_output_count: usize,
    floater_filter_removed_count: usize,
    fill_ms: u128,
    carve_ms: u128,
    mesh_ms: u128,
    voxel_solid_cells: usize,
    filled_solid_cells: usize,
    carved_solid_cells: usize,
    cropped_solid_cells: usize,
    voxel_grid_dims: [usize; 3],
    filled_grid_dims: [usize; 3],
    carved_grid_dims: [usize; 3],
    cropped_grid_dims: [usize; 3],
    crop_min_cell: [usize; 3],
    crop_max_cell: [usize; 3],
    carve_reachable_cells: usize,
    carve_requested_seed: [f32; 3],
    carve_resolved_seed: Option<[f32; 3]>,
    collision_warnings: Vec<String>,
}

pub fn process_file(
    input_path: impl AsRef<Path>,
    out_dir: impl AsRef<Path>,
    config: &ProcessConfig,
    recipe: &RecipeBundle,
) -> AgResult<ProcessOutput> {
    process_file_with_cancel(input_path, out_dir, config, recipe, || false)
}

pub fn process_file_with_cancel(
    input_path: impl AsRef<Path>,
    out_dir: impl AsRef<Path>,
    config: &ProcessConfig,
    recipe: &RecipeBundle,
    should_cancel: impl Fn() -> bool,
) -> AgResult<ProcessOutput> {
    process_file_with_cancel_and_progress(
        input_path,
        out_dir,
        config,
        recipe,
        should_cancel,
        |_| {},
        None,
    )
}

pub fn process_file_with_cancel_and_progress(
    input_path: impl AsRef<Path>,
    out_dir: impl AsRef<Path>,
    config: &ProcessConfig,
    recipe: &RecipeBundle,
    should_cancel: impl Fn() -> bool,
    progress: impl Fn(&str),
    wgpu_ctx: Option<&crate::gpu::WgpuContext>,
) -> AgResult<ProcessOutput> {
    let input_path = input_path.as_ref();
    let out_dir = out_dir.as_ref();
    fs::create_dir_all(out_dir)?;
    harden_permissions(out_dir)?;

    progress("decode");
    let source_bytes = fs::metadata(input_path)
        .map(|m| m.len())
        .unwrap_or_default();
    let start = Instant::now();
    let (mut table, format) = read_source(input_path)?;
    let decode_ms = start.elapsed().as_millis();
    let source_count = table.len();
    check_cancelled(&should_cancel)?;

    progress("alignment");
    let start = Instant::now();
    let alignment_transform = bake_alignment(&mut table, recipe.alignment_recipe.as_ref())?;
    let alignment_ms = start.elapsed().as_millis();
    let aligned_carve_config = aligned_voxel_carve_config(&config.voxel_carve, alignment_transform);
    progress("filters");
    table = filter_nan(&table)?;
    let mut collision_warnings = Vec::new();
    let mut filter_cluster_input_count = 0usize;
    let mut filter_cluster_output_count = 0usize;
    let mut filter_cluster_removed_count = 0usize;
    let mut filter_cluster_requested_seed = None;
    let mut filter_cluster_resolved_seed = None;
    let mut filter_cluster_seed_resolved = false;
    let mut filter_cluster_occupied_cells = 0usize;
    let mut filter_cluster_cells = 0usize;
    if let Some(edit_recipe) = &recipe.edit_recipe {
        for op in &edit_recipe.operations {
            match op {
                EditOperation::SelectAll
                | EditOperation::SelectNone
                | EditOperation::FilterFloatersByVoxelContribution { .. } => {}
                EditOperation::FilterOpacity { min } => {
                    table = filter_opacity_min(&table, *min)?;
                }
                EditOperation::FilterBox { min, max } => {
                    let (min, max) = aligned_filter_box(alignment_transform, *min, *max);
                    table = filter_box(&table, min, max)?;
                }
                EditOperation::FilterSphere { center, radius } => {
                    let center = alignment_transform.apply_point(Vec3::from_array(*center));
                    table = filter_sphere(&table, center, *radius * alignment_transform.scale)?;
                }
                EditOperation::FilterCluster {
                    coarse_voxel_size,
                    opacity_threshold,
                    seed_pos,
                    min_contribution,
                } => {
                    let outcome = filter_cluster_with_stats(
                        &table,
                        *coarse_voxel_size,
                        *opacity_threshold,
                        alignment_transform.apply_point(Vec3::from_array(*seed_pos)),
                        *min_contribution,
                    )?;
                    filter_cluster_input_count += outcome.input_count;
                    filter_cluster_output_count = outcome.output_count;
                    filter_cluster_removed_count += outcome.removed_count;
                    filter_cluster_requested_seed = Some(outcome.requested_seed);
                    filter_cluster_resolved_seed = Some(outcome.resolved_seed);
                    filter_cluster_seed_resolved |= outcome.seed_was_resolved;
                    filter_cluster_occupied_cells = outcome.occupied_cells;
                    filter_cluster_cells = outcome.cluster_cells;
                    if outcome.seed_was_resolved {
                        collision_warnings.push(format!(
                            "filterCluster seed resolved from {:?} to nearest occupied voxel {:?}",
                            outcome.requested_seed, outcome.resolved_seed
                        ));
                    }
                    table = outcome.table;
                }
            }
        }
    }
    check_cancelled(&should_cancel)?;

    let mut reconstruction = match config.reconstruction.method {
        ReconstructionMethod::Voxel => reconstruct_with_voxel(
            table,
            config,
            recipe,
            aligned_carve_config,
            collision_warnings,
            &should_cancel,
            &progress,
            wgpu_ctx,
        )?,
        ReconstructionMethod::Sugar | ReconstructionMethod::Poisson => {
            reconstruct_with_external_adapter(
                table,
                out_dir,
                config,
                collision_warnings,
                &should_cancel,
                &progress,
            )?
        }
    };
    let collision_mesh = reconstruction.collision_mesh.clone();
    let table = reconstruction.table;
    let triangle_count = collision_mesh.triangle_count();
    let geometric_error = mesh_error_against_splat_centers(&table, &collision_mesh);
    check_cancelled(&should_cancel)?;

    progress("navmesh");
    let start = Instant::now();
    let navmesh = bake_navmesh(&collision_mesh, &config.navmesh)?;
    let navmesh_ms = start.elapsed().as_millis();
    check_cancelled(&should_cancel)?;
    let mesh_path = out_dir.join("collision_mesh.json");
    let occlusion_glb_path = out_dir.join("occlusion.glb");
    let scene_name = "scene.sog".to_string();
    let scene_path = out_dir.join(&scene_name);
    let mut navmesh_glb_name = None;
    let mut navmesh_bin_name = None;
    let navmesh_triangle_count = navmesh.as_ref().map(|m| m.triangle_count()).unwrap_or(0);
    let navmesh_glb_path = out_dir.join("navmesh.glb");
    let navmesh_bin_path = out_dir.join("navmesh.bin");
    if config.navmesh.enabled && navmesh_triangle_count == 0 {
        reconstruction.collision_warnings.push(
            "navmesh generated 0 triangles; likely causes are a blocked seed, empty carve result, wrong floor plane, or wrong up axis"
                .to_string(),
        );
    }
    if navmesh.is_some() {
        navmesh_glb_name = Some(file_name(&navmesh_glb_path));
        navmesh_bin_name = Some(file_name(&navmesh_bin_path));
    } else {
        remove_if_exists(&navmesh_glb_path)?;
        remove_if_exists(&navmesh_bin_path)?;
    }

    progress("export");
    let start = Instant::now();
    fs::write(&mesh_path, serde_json::to_vec_pretty(&collision_mesh)?)?;
    write_mesh_glb(&occlusion_glb_path, &collision_mesh, "GA3D_OCCLUSION")?;
    write_sog_bundle(&scene_path, &table)?;
    if let Some(navmesh) = &navmesh {
        write_mesh_glb(&navmesh_glb_path, navmesh, "GA3D_NAVMESH")?;
        write_navmesh_bin(&navmesh_bin_path, navmesh)?;
    }
    let scene_sog_bytes = file_size(&scene_path);
    let optimized_glb_bytes = file_size(&occlusion_glb_path);

    let mut manifest = Manifest {
        version: 1,
        schema_version: 2,
        output_dir: out_dir.to_string_lossy().to_string(),
        source: SourceStats {
            format,
            splat_count: source_count,
            kept_count: table.len(),
            ..SourceStats::default()
        },
        alignment: AlignmentManifest {
            unit_scale: alignment_transform.scale,
            origin: alignment_transform.origin.to_array(),
            rotation_quat_wxyz: [
                alignment_transform.rotation.w,
                alignment_transform.rotation.x,
                alignment_transform.rotation.y,
                alignment_transform.rotation.z,
            ],
        },
        bounds: Some(table.scene_bounds()),
        artifacts: ArtifactManifest {
            manifest: "manifest.json".to_string(),
            index_html: "index.html".to_string(),
            scene: scene_name.clone(),
            collision_mesh_json: file_name(&mesh_path),
            occlusion_glb: file_name(&occlusion_glb_path),
            navmesh_glb: navmesh_glb_name.clone(),
            navmesh_bin: navmesh_bin_name.clone(),
            webar_zip: config
                .export
                .write_webar_zip
                .then(|| "webar.zip".to_string()),
        },
        metrics: Metrics {
            reconstruction_method: reconstruction.reconstruction_method,
            reconstruction_ms: reconstruction.reconstruction_ms,
            reconstruction_adapter_status: reconstruction.reconstruction_adapter_status,
            decode_ms,
            alignment_ms,
            voxel_ms: reconstruction.voxel_ms,
            cpu_voxel_ms: reconstruction.cpu_voxel_ms,
            gpu_voxel_ms: reconstruction.gpu_voxel_ms,
            gpu_voxel_speedup: reconstruction.gpu_voxel_speedup,
            voxel_backend: reconstruction.voxel_backend,
            cpu_gpu_voxel_mismatches: reconstruction.cpu_gpu_voxel_mismatches,
            filter_cluster_input_count,
            filter_cluster_output_count,
            filter_cluster_removed_count,
            filter_cluster_requested_seed,
            filter_cluster_resolved_seed,
            filter_cluster_seed_resolved,
            filter_cluster_occupied_cells,
            filter_cluster_cells,
            floater_filter_input_count: reconstruction.floater_filter_input_count,
            floater_filter_output_count: reconstruction.floater_filter_output_count,
            floater_filter_removed_count: reconstruction.floater_filter_removed_count,
            fill_ms: reconstruction.fill_ms,
            carve_ms: reconstruction.carve_ms,
            mesh_ms: reconstruction.mesh_ms,
            navmesh_ms,
            export_ms: 0,
            voxel_solid_cells: reconstruction.voxel_solid_cells,
            filled_solid_cells: reconstruction.filled_solid_cells,
            carved_solid_cells: reconstruction.carved_solid_cells,
            cropped_solid_cells: reconstruction.cropped_solid_cells,
            voxel_grid_dims: reconstruction.voxel_grid_dims,
            filled_grid_dims: reconstruction.filled_grid_dims,
            carved_grid_dims: reconstruction.carved_grid_dims,
            cropped_grid_dims: reconstruction.cropped_grid_dims,
            crop_min_cell: reconstruction.crop_min_cell,
            crop_max_cell: reconstruction.crop_max_cell,
            carve_reachable_cells: reconstruction.carve_reachable_cells,
            carve_requested_seed: reconstruction.carve_requested_seed,
            carve_resolved_seed: reconstruction.carve_resolved_seed,
            collision_warnings: reconstruction.collision_warnings,
            source_bytes,
            scene_sog_bytes,
            optimized_glb_bytes,
            source_to_optimized_glb_ratio: size_ratio(source_bytes, optimized_glb_bytes),
            optimized_glb_to_source_ratio: size_ratio(optimized_glb_bytes, source_bytes),
            source_to_webar_zip_ratio: None,
            webar_zip_to_source_ratio: None,
            collision_triangles_before_merge: collision_mesh.triangles_before_merge,
            collision_triangles_after_merge: triangle_count,
            navmesh_triangles: navmesh_triangle_count,
            webar_zip_bytes: 0,
            geometric_error_sample_count: geometric_error.sample_count,
            geometric_error_mean: geometric_error.mean,
            geometric_error_rms: geometric_error.rms,
            geometric_error_p95: geometric_error.p95,
        },
    };

    let manifest_path = out_dir.join("manifest.json");
    let index_html_path = out_dir.join("index.html");
    let asset_js_dir = out_dir.join("assets").join("js");
    let playcanvas_path = asset_js_dir.join("playcanvas.min.js");
    let playcanvas_license_path = asset_js_dir.join("playcanvas.LICENSE.txt");
    fs::create_dir_all(&asset_js_dir)?;
    fs::write(&playcanvas_path, crate::webar::playcanvas_runtime())?;
    fs::write(&playcanvas_license_path, crate::webar::playcanvas_license())?;
    fs::write(&index_html_path, crate::webar::viewer_html())?;
    let mut files = vec![
        ("index.html", index_html_path.clone()),
        ("assets/js/playcanvas.min.js", playcanvas_path.clone()),
        (
            "assets/js/playcanvas.LICENSE.txt",
            playcanvas_license_path.clone(),
        ),
        (scene_name.as_str(), scene_path.clone()),
        ("collision_mesh.json", mesh_path.clone()),
        ("occlusion.glb", occlusion_glb_path.clone()),
    ];
    if let Some(name) = &navmesh_glb_name {
        files.push((name.as_str(), out_dir.join(name)));
    }
    if let Some(name) = &navmesh_bin_name {
        files.push((name.as_str(), out_dir.join(name)));
    }
    let webar_zip_path = out_dir.join("webar.zip");
    if config.export.write_webar_zip {
        for _ in 0..10 {
            fs::write(&manifest_path, serde_json::to_vec_pretty(&manifest)?)?;
            write_webar_zip(&webar_zip_path, &manifest, &files)?;
            let size = fs::metadata(&webar_zip_path).map(|m| m.len()).unwrap_or(0);
            let elapsed = start.elapsed().as_millis();
            if size == manifest.metrics.webar_zip_bytes {
                manifest.metrics.export_ms = elapsed;
                break;
            }
            manifest.metrics.webar_zip_bytes = size;
            manifest.metrics.source_to_webar_zip_ratio = size_ratio(source_bytes, size);
            manifest.metrics.webar_zip_to_source_ratio = size_ratio(size, source_bytes);
            manifest.metrics.export_ms = elapsed;
        }
        fs::write(&manifest_path, serde_json::to_vec_pretty(&manifest)?)?;
        write_webar_zip(&webar_zip_path, &manifest, &files)?;
    } else {
        remove_if_exists(&webar_zip_path)?;
        manifest.metrics.export_ms = start.elapsed().as_millis();
        fs::write(&manifest_path, serde_json::to_vec_pretty(&manifest)?)?;
    }

    progress("done");
    Ok(ProcessOutput {
        manifest,
        collision_mesh,
        navmesh,
    })
}

fn reconstruct_with_voxel(
    mut table: crate::SplatTable,
    config: &ProcessConfig,
    recipe: &RecipeBundle,
    aligned_carve_config: VoxelCarveConfig,
    mut collision_warnings: Vec<String>,
    should_cancel: &impl Fn() -> bool,
    progress: &impl Fn(&str),
    wgpu_ctx: Option<&crate::gpu::WgpuContext>,
) -> AgResult<ReconstructionOutcome> {
    let reconstruction_start = Instant::now();
    let mut floater_filter_input_count = 0;
    let mut floater_filter_output_count = 0;
    let mut floater_filter_removed_count = 0;
    progress("voxelize");
    let voxel_params = VoxelParams {
        size: config.voxel.size,
        opacity_threshold: config.voxel.opacity_threshold,
    };
    let start = Instant::now();
    let mut cpu_grid = voxelize_cpu(&table, voxel_params)?;
    if let Some(min_contribution) = recipe.edit_recipe.as_ref().and_then(|recipe| {
        recipe.operations.iter().find_map(|op| {
            if let EditOperation::FilterFloatersByVoxelContribution { min_contribution } = op {
                Some(*min_contribution)
            } else {
                None
            }
        })
    }) {
        let outcome = filter_floaters_by_voxel_contribution_with_stats(
            &table,
            &cpu_grid,
            voxel_params,
            min_contribution,
        )?;
        floater_filter_input_count += outcome.input_count;
        floater_filter_output_count = outcome.output_count;
        floater_filter_removed_count += outcome.removed_count;
        table = outcome.table;
        cpu_grid = voxelize_cpu(&table, voxel_params)?;
    }
    let cpu_voxel_ms = start.elapsed().as_millis();
    check_cancelled(should_cancel)?;
    let mut gpu_voxel_ms = None;
    let mut cpu_gpu_voxel_mismatches = None;
    let (grid, voxel_ms, voxel_backend) = match config.voxel.backend {
        VoxelBackend::Cpu => {
            if config.voxel.compare_cpu_gpu {
                let start = Instant::now();
                let gpu_grid = voxelize_gpu_blocking(&table, voxel_params, wgpu_ctx)?;
                gpu_voxel_ms = Some(start.elapsed().as_millis());
                cpu_gpu_voxel_mismatches = Some(cpu_grid.mismatch_count(&gpu_grid));
                check_cancelled(should_cancel)?;
            }
            (cpu_grid, cpu_voxel_ms, "cpu".to_string())
        }
        VoxelBackend::Gpu => {
            let start = Instant::now();
            let gpu_grid = voxelize_gpu_blocking(&table, voxel_params, wgpu_ctx)?;
            let elapsed = start.elapsed().as_millis();
            gpu_voxel_ms = Some(elapsed);
            let mismatches = cpu_grid.mismatch_count(&gpu_grid);
            if config.voxel.compare_cpu_gpu {
                cpu_gpu_voxel_mismatches = Some(mismatches);
            }
            (gpu_grid, elapsed, "gpu".to_string())
        }
    };
    check_cancelled(should_cancel)?;

    progress("fill");
    let start = Instant::now();
    let fill_outcome = fill_grid_with_status(
        &grid,
        &config.voxel_fill,
        Vec3::from_array(aligned_carve_config.seed_pos),
    );
    let fill_ms = start.elapsed().as_millis();
    if let Some(warning) = &fill_outcome.warning {
        collision_warnings.push(warning.clone());
    }
    let filled_solid_cells = fill_outcome.after_solid;
    let filled = fill_outcome.grid;
    check_cancelled(should_cancel)?;

    progress("carve");
    let start = Instant::now();
    let carve_outcome = carve_grid_with_status(&filled, &aligned_carve_config);
    let carve_ms = start.elapsed().as_millis();
    if let Some(warning) = &carve_outcome.warning {
        collision_warnings.push(warning.clone());
    }
    let carve_reachable_cells = carve_outcome.reachable_cells;
    let carve_requested_seed = carve_outcome.requested_seed;
    let carve_resolved_seed = carve_outcome.resolved_seed;
    let carved_solid_cells = carve_outcome.after_solid;
    let carved = carve_outcome.grid;
    check_cancelled(should_cancel)?;

    progress("mesh");
    let start = Instant::now();
    let (collision_grid, crop_stats) = carved.crop_to_occupied();
    let collision_mesh = extract_mesh(&collision_grid, config.mesh.mode)?;
    let mesh_ms = start.elapsed().as_millis();
    if collision_mesh.triangle_count() == 0 {
        collision_warnings.push(
            "collision mesh generated 0 triangles; check voxel size, opacity threshold, bake profile, or carve/fill seed"
                .to_string(),
        );
    }

    Ok(ReconstructionOutcome {
        collision_mesh,
        table,
        reconstruction_method: ReconstructionMethod::Voxel.as_str().to_string(),
        reconstruction_ms: reconstruction_start.elapsed().as_millis(),
        reconstruction_adapter_status: None,
        gpu_voxel_ms,
        cpu_gpu_voxel_mismatches,
        gpu_voxel_speedup: gpu_voxel_ms.and_then(|gpu| {
            if gpu == 0 {
                None
            } else {
                Some(cpu_voxel_ms as f64 / gpu as f64)
            }
        }),
        voxel_backend,
        voxel_ms,
        cpu_voxel_ms,
        floater_filter_input_count,
        floater_filter_output_count,
        floater_filter_removed_count,
        fill_ms,
        carve_ms,
        mesh_ms,
        voxel_solid_cells: grid.solid_count(),
        filled_solid_cells,
        carved_solid_cells,
        cropped_solid_cells: collision_grid.solid_count(),
        voxel_grid_dims: grid.dims,
        filled_grid_dims: filled.dims,
        carved_grid_dims: carved.dims,
        cropped_grid_dims: collision_grid.dims,
        crop_min_cell: crop_stats.min_cell,
        crop_max_cell: crop_stats.max_cell,
        carve_reachable_cells,
        carve_requested_seed,
        carve_resolved_seed,
        collision_warnings,
    })
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AdapterRequest {
    input_ply: String,
    out_dir: String,
    method: String,
    target_triangles: u32,
    timeout_seconds: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AdapterResponse {
    mesh_json: PathBuf,
    #[serde(default)]
    warnings: Vec<String>,
}

fn reconstruct_with_external_adapter(
    table: crate::SplatTable,
    out_dir: &Path,
    config: &ProcessConfig,
    mut collision_warnings: Vec<String>,
    should_cancel: &impl Fn() -> bool,
    progress: &impl Fn(&str),
) -> AgResult<ReconstructionOutcome> {
    progress("reconstruct");
    let reconstruction_start = Instant::now();
    let adapter_command = config
        .reconstruction
        .adapter_command
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| {
            AgError::InvalidConfig(
                "adapterCommand is required when reconstruction method is not voxel".to_string(),
            )
        })?;
    let adapter_dir = out_dir.join("adapter-work");
    fs::create_dir_all(&adapter_dir)?;
    harden_permissions(&adapter_dir)?;
    let input_ply = adapter_dir.join("filtered-input.ply");
    write_ply(&input_ply, &table)?;
    let request = AdapterRequest {
        input_ply: input_ply.to_string_lossy().to_string(),
        out_dir: out_dir.to_string_lossy().to_string(),
        method: config.reconstruction.method.as_str().to_string(),
        target_triangles: config.reconstruction.target_triangles,
        timeout_seconds: config.reconstruction.timeout_seconds,
    };
    let stdout = run_adapter(
        adapter_command,
        &adapter_dir,
        &serde_json::to_vec(&request)?,
        Duration::from_secs(config.reconstruction.timeout_seconds as u64),
        should_cancel,
    )?;
    let response: AdapterResponse = serde_json::from_slice(stdout.as_bytes())?;
    collision_warnings.extend(response.warnings);
    let mesh_path = resolve_adapter_mesh_path(out_dir, &response.mesh_json)?;
    harden_permissions(&mesh_path)?;
    let collision_mesh: Mesh = serde_json::from_slice(&fs::read(&mesh_path)?)?;
    validate_mesh(&collision_mesh)?;
    warn_if_mesh_exceeds_target(&collision_mesh, config, &mut collision_warnings);

    Ok(ReconstructionOutcome {
        collision_mesh,
        table,
        reconstruction_method: config.reconstruction.method.as_str().to_string(),
        reconstruction_ms: reconstruction_start.elapsed().as_millis(),
        reconstruction_adapter_status: Some("ok".to_string()),
        gpu_voxel_ms: None,
        cpu_gpu_voxel_mismatches: None,
        gpu_voxel_speedup: None,
        voxel_backend: "external".to_string(),
        voxel_ms: 0,
        cpu_voxel_ms: 0,
        floater_filter_input_count: 0,
        floater_filter_output_count: 0,
        floater_filter_removed_count: 0,
        fill_ms: 0,
        carve_ms: 0,
        mesh_ms: 0,
        voxel_solid_cells: 0,
        filled_solid_cells: 0,
        carved_solid_cells: 0,
        cropped_solid_cells: 0,
        voxel_grid_dims: [0, 0, 0],
        filled_grid_dims: [0, 0, 0],
        carved_grid_dims: [0, 0, 0],
        cropped_grid_dims: [0, 0, 0],
        crop_min_cell: [0, 0, 0],
        crop_max_cell: [0, 0, 0],
        carve_reachable_cells: 0,
        carve_requested_seed: config.voxel_carve.seed_pos,
        carve_resolved_seed: None,
        collision_warnings,
    })
}

fn run_adapter(
    adapter_command: &str,
    working_dir: &Path,
    stdin_json: &[u8],
    timeout: Duration,
    should_cancel: &impl Fn() -> bool,
) -> AgResult<String> {
    let parts = split_command(adapter_command)?;
    let program = parts
        .first()
        .ok_or_else(|| AgError::InvalidConfig("adapterCommand must not be empty".to_string()))?;
    let mut child = Command::new(program)
        .args(&parts[1..])
        .current_dir(working_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| {
            AgError::InvalidConfig(format!("failed to start reconstruction adapter: {err}"))
        })?;
    child.stdin.take().unwrap().write_all(stdin_json)?;
    let stdout = Arc::new(Mutex::new(String::new()));
    let stderr = Arc::new(Mutex::new(String::new()));
    let out_reader = spawn_reader(child.stdout.take(), stdout.clone());
    let err_reader = spawn_reader(child.stderr.take(), stderr.clone());
    let start = Instant::now();
    loop {
        check_cancelled(should_cancel)?;
        if let Some(status) = child.try_wait()? {
            out_reader.join().ok();
            err_reader.join().ok();
            let out = stdout.lock().unwrap().clone();
            let err = stderr.lock().unwrap().clone();
            if !status.success() {
                return Err(AgError::InvalidInput(format!(
                    "reconstruction adapter exited with {status}: {}",
                    err.trim()
                )));
            }
            return Ok(out);
        }
        if start.elapsed() > timeout {
            let _ = child.kill();
            return Err(AgError::InvalidInput(
                "reconstruction adapter timed out".to_string(),
            ));
        }
        thread::sleep(Duration::from_millis(20));
    }
}

fn spawn_reader<R: Read + Send + 'static>(
    reader: Option<R>,
    output: Arc<Mutex<String>>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        if let Some(mut reader) = reader {
            let mut text = String::new();
            let _ = reader.read_to_string(&mut text);
            *output.lock().unwrap() = text;
        }
    })
}

fn resolve_adapter_mesh_path(out_dir: &Path, mesh_path: &Path) -> AgResult<PathBuf> {
    let candidate = if mesh_path.is_absolute() {
        mesh_path.to_path_buf()
    } else {
        out_dir.join(mesh_path)
    };
    let out_root = fs::canonicalize(out_dir)?;
    let canonical = fs::canonicalize(&candidate)?;
    if !canonical.starts_with(&out_root) {
        return Err(AgError::InvalidInput(
            "adapter mesh output must stay inside resolved outDir".to_string(),
        ));
    }
    Ok(canonical)
}

fn warn_if_mesh_exceeds_target(
    mesh: &Mesh,
    config: &ProcessConfig,
    collision_warnings: &mut Vec<String>,
) {
    let ideal_limit = (config.reconstruction.target_triangles as f32 * 1.2).ceil() as usize;
    if mesh.triangle_count() > ideal_limit {
        collision_warnings.push(format!(
            "reconstruction adapter mesh has {} triangles, above targetTriangles * 1.2 ({ideal_limit})",
            mesh.triangle_count()
        ));
    }
}

fn split_command(command: &str) -> AgResult<Vec<String>> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    for ch in command.chars() {
        match ch {
            '"' => in_quotes = !in_quotes,
            ' ' | '\t' if !in_quotes => {
                if !current.is_empty() {
                    parts.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(ch),
        }
    }
    if in_quotes {
        return Err(AgError::InvalidConfig(
            "adapterCommand contains an unterminated quote".to_string(),
        ));
    }
    if !current.is_empty() {
        parts.push(current);
    }
    Ok(parts)
}

#[cfg(unix)]
fn harden_permissions(path: &Path) -> AgResult<()> {
    use std::os::unix::fs::PermissionsExt;
    let metadata = fs::metadata(path)?;
    let mut permissions = metadata.permissions();
    let mode = permissions.mode();
    if mode & 0o002 != 0 {
        let target = if metadata.is_dir() { 0o755 } else { 0o644 };
        permissions.set_mode(target);
        fs::set_permissions(path, permissions)?;
    }
    Ok(())
}

#[cfg(not(unix))]
fn harden_permissions(_path: &Path) -> AgResult<()> {
    Ok(())
}

fn check_cancelled(should_cancel: &impl Fn() -> bool) -> AgResult<()> {
    if should_cancel() {
        Err(AgError::Cancelled)
    } else {
        Ok(())
    }
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_string()
}

fn file_size(path: &Path) -> u64 {
    fs::metadata(path).map(|m| m.len()).unwrap_or(0)
}

fn size_ratio(numerator: u64, denominator: u64) -> Option<f64> {
    if numerator == 0 || denominator == 0 {
        None
    } else {
        Some(numerator as f64 / denominator as f64)
    }
}

fn remove_if_exists(path: &Path) -> AgResult<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err.into()),
    }
}

fn aligned_voxel_carve_config(
    config: &VoxelCarveConfig,
    transform: AlignmentTransform,
) -> VoxelCarveConfig {
    let mut aligned = config.clone();
    aligned.seed_pos = transform
        .apply_point(Vec3::from_array(config.seed_pos))
        .to_array();
    aligned
}

fn aligned_filter_box(transform: AlignmentTransform, min: [f32; 3], max: [f32; 3]) -> (Vec3, Vec3) {
    let min = Vec3::from_array(min);
    let max = Vec3::from_array(max);
    let mut bounds = Bounds::empty();
    for x in [min.x, max.x] {
        for y in [min.y, max.y] {
            for z in [min.z, max.z] {
                bounds.include(transform.apply_point(Vec3::new(x, y, z)));
            }
        }
    }
    (bounds.min, bounds.max)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AlignmentRecipe, EditRecipe, FillMode, MeshMode};

    #[test]
    fn optional_artifacts_are_absent_when_manifest_says_absent() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("scene.splat");
        fs::write(&input, single_splat_bytes()).unwrap();
        let out = dir.path().join("out");
        fs::create_dir_all(&out).unwrap();
        fs::write(out.join("navmesh.glb"), b"stale").unwrap();
        fs::write(out.join("navmesh.bin"), b"stale").unwrap();
        fs::write(out.join("webar.zip"), b"stale").unwrap();

        let mut config = ProcessConfig::default();
        config.voxel.size = 0.25;
        config.voxel.opacity_threshold = 0.05;
        config.mesh.mode = MeshMode::Faces;
        config.navmesh.enabled = false;
        config.export.write_webar_zip = false;

        let output = process_file(&input, &out, &config, &RecipeBundle::default()).unwrap();

        assert!(output.manifest.artifacts.navmesh_glb.is_none());
        assert!(output.manifest.artifacts.navmesh_bin.is_none());
        assert!(output.manifest.artifacts.webar_zip.is_none());
        assert!(!out.join("navmesh.glb").exists());
        assert!(!out.join("navmesh.bin").exists());
        assert!(!out.join("webar.zip").exists());
    }

    #[test]
    fn alignment_is_applied_to_downstream_seed_coordinates() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("scene.splat");
        fs::write(&input, splat_bytes_at(&[[2.0, 0.0, 0.0], [4.0, 0.0, 0.0]])).unwrap();
        let out = dir.path().join("out");

        let mut config = ProcessConfig::default();
        config.voxel.size = 0.25;
        config.voxel.opacity_threshold = 0.05;
        config.voxel_fill.mode = FillMode::None;
        config.voxel_carve.enabled = false;
        config.mesh.mode = MeshMode::Faces;
        config.navmesh.enabled = false;
        config.export.write_webar_zip = false;

        let recipe = RecipeBundle {
            alignment_recipe: Some(AlignmentRecipe {
                up_axis: None,
                floor_normal: None,
                scale_points: Some([[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]]),
                scale_distance_meters: Some(1.0),
                origin: None,
            }),
            edit_recipe: Some(EditRecipe {
                operations: vec![EditOperation::FilterCluster {
                    coarse_voxel_size: 0.25,
                    opacity_threshold: 0.05,
                    seed_pos: [2.0, 0.0, 0.0],
                    min_contribution: 0.05,
                }],
            }),
        };

        let output = process_file(&input, &out, &config, &recipe).unwrap();

        assert_eq!(output.manifest.source.kept_count, 1);
        let requested_seed = output
            .manifest
            .metrics
            .filter_cluster_requested_seed
            .unwrap();
        assert!((requested_seed[0] - 1.0).abs() < 1e-6);
        let bounds = output.manifest.bounds.unwrap();
        let center_x = (bounds.min.x + bounds.max.x) * 0.5;
        assert!(
            (center_x - 1.0).abs() < 0.05,
            "expected the source-space seed to select the first aligned cluster, got {bounds:?}"
        );
    }

    #[test]
    fn object_style_config_keeps_all_splats_without_cluster_filter() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("object.splat");
        fs::write(
            &input,
            splat_bytes_at(&[[0.0, 0.0, 0.0], [0.2, 0.0, 0.0], [0.0, 0.2, 0.0]]),
        )
        .unwrap();
        let out = dir.path().join("out");

        let mut config = ProcessConfig::default();
        config.voxel.size = 0.05;
        config.voxel.opacity_threshold = 0.1;
        config.voxel_fill.mode = FillMode::None;
        config.voxel_fill.dilation_size = 0.0;
        config.voxel_carve.enabled = false;
        config.mesh.mode = MeshMode::Smooth;
        config.navmesh.enabled = false;
        config.export.write_webar_zip = false;

        let output = process_file(&input, &out, &config, &RecipeBundle::default()).unwrap();

        assert_eq!(output.manifest.source.splat_count, 3);
        assert_eq!(output.manifest.source.kept_count, 3);
        assert_eq!(output.manifest.metrics.filter_cluster_input_count, 0);
        assert_eq!(output.manifest.metrics.filter_cluster_removed_count, 0);
        assert!(output.manifest.metrics.collision_triangles_after_merge > 0);
        assert!(
            output
                .manifest
                .metrics
                .collision_warnings
                .iter()
                .all(|warning| !warning.contains("navmesh generated 0 triangles"))
        );
    }

    fn single_splat_bytes() -> Vec<u8> {
        splat_bytes_at(&[[0.0, 0.0, 0.0]])
    }

    fn splat_bytes_at(points: &[[f32; 3]]) -> Vec<u8> {
        let mut bytes = Vec::new();
        for point in points {
            bytes.extend_from_slice(&point[0].to_le_bytes());
            bytes.extend_from_slice(&point[1].to_le_bytes());
            bytes.extend_from_slice(&point[2].to_le_bytes());
            bytes.extend_from_slice(&0.3f32.to_le_bytes());
            bytes.extend_from_slice(&0.3f32.to_le_bytes());
            bytes.extend_from_slice(&0.3f32.to_le_bytes());
            bytes.extend_from_slice(&[128, 128, 128, 255, 255, 128, 128, 128]);
        }
        bytes
    }
}
