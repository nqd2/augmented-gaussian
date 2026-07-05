use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessConfig {
    #[serde(default)]
    pub reconstruction: ReconstructionConfig,
    #[serde(default)]
    pub voxel: VoxelConfig,
    #[serde(default)]
    pub voxel_fill: VoxelFillConfig,
    #[serde(default)]
    pub voxel_carve: VoxelCarveConfig,
    #[serde(default)]
    pub mesh: MeshConfig,
    #[serde(default)]
    pub navmesh: NavmeshConfig,
    #[serde(default)]
    pub export: ExportConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ReconstructionConfig {
    #[serde(default)]
    pub method: ReconstructionMethod,
    #[serde(default)]
    pub adapter_command: Option<String>,
    #[serde(default = "default_target_tris")]
    pub target_triangles: u32,
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u32,
}

impl Default for ReconstructionConfig {
    fn default() -> Self {
        Self {
            method: ReconstructionMethod::Voxel,
            adapter_command: None,
            target_triangles: default_target_tris(),
            timeout_seconds: default_timeout(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ReconstructionMethod {
    Voxel,
    Sugar,
    Poisson,
}

impl Default for ReconstructionMethod {
    fn default() -> Self {
        Self::Voxel
    }
}

impl ReconstructionMethod {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Voxel => "voxel",
            Self::Sugar => "sugar",
            Self::Poisson => "poisson",
        }
    }
}

fn default_target_tris() -> u32 {
    50_000
}

fn default_timeout() -> u32 {
    900
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoxelConfig {
    #[serde(default = "default_voxel_backend")]
    pub backend: VoxelBackend,
    #[serde(default)]
    pub compare_cpu_gpu: bool,
    pub size: f32,
    pub opacity_threshold: f32,
}

impl Default for VoxelConfig {
    fn default() -> Self {
        Self {
            backend: VoxelBackend::Cpu,
            compare_cpu_gpu: false,
            size: 0.1,
            opacity_threshold: 0.1,
        }
    }
}

fn default_voxel_backend() -> VoxelBackend {
    VoxelBackend::Cpu
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum VoxelBackend {
    Cpu,
    Gpu,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoxelFillConfig {
    pub mode: FillMode,
    pub dilation_size: f32,
}

impl Default for VoxelFillConfig {
    fn default() -> Self {
        Self {
            mode: FillMode::None,
            dilation_size: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum FillMode {
    None,
    ExteriorFill,
    FloorFill,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoxelCarveConfig {
    pub enabled: bool,
    pub agent_height: f32,
    pub agent_radius: f32,
    pub seed_pos: [f32; 3],
}

impl Default for VoxelCarveConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            agent_height: 1.6,
            agent_radius: 0.2,
            seed_pos: [0.0, 0.0, 0.0],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MeshConfig {
    pub mode: MeshMode,
}

impl Default for MeshConfig {
    fn default() -> Self {
        Self {
            mode: MeshMode::Smooth,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum MeshMode {
    Faces,
    Smooth,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NavmeshConfig {
    pub enabled: bool,
    pub agent_height: f32,
    pub agent_radius: f32,
    pub max_slope_degrees: f32,
    pub cell_size: f32,
    pub cell_height: f32,
    pub walkable_climb: f32,
    pub min_region_size: u16,
    pub merge_region_size: u16,
}

impl Default for NavmeshConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            agent_height: 1.6,
            agent_radius: 0.2,
            max_slope_degrees: 45.0,
            cell_size: 0.1,
            cell_height: 0.05,
            walkable_climb: 0.25,
            min_region_size: 4,
            merge_region_size: 12,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportConfig {
    pub write_webar_zip: bool,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            write_webar_zip: true,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecipeBundle {
    #[serde(default)]
    pub alignment_recipe: Option<AlignmentRecipe>,
    #[serde(default)]
    pub edit_recipe: Option<EditRecipe>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlignmentRecipe {
    #[serde(default)]
    pub up_axis: Option<UpAxis>,
    pub floor_normal: Option<[f32; 3]>,
    pub scale_points: Option<[[f32; 3]; 2]>,
    pub scale_distance_meters: Option<f32>,
    pub origin: Option<[f32; 3]>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum UpAxis {
    X,
    Y,
    Z,
    NegX,
    NegY,
    NegZ,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditRecipe {
    #[serde(default)]
    pub operations: Vec<EditOperation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum EditOperation {
    SelectAll,
    SelectNone,
    FilterOpacity {
        min: f32,
    },
    FilterBox {
        min: [f32; 3],
        max: [f32; 3],
    },
    FilterSphere {
        center: [f32; 3],
        radius: f32,
    },
    FilterCluster {
        coarse_voxel_size: f32,
        opacity_threshold: f32,
        seed_pos: [f32; 3],
        #[serde(default = "default_filter_min_contribution")]
        min_contribution: f32,
    },
    FilterFloatersByVoxelContribution {
        #[serde(default = "default_filter_min_contribution")]
        min_contribution: f32,
    },
}

fn default_filter_min_contribution() -> f32 {
    0.1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_process_config_defaults_to_voxel_reconstruction() {
        let config: ProcessConfig = serde_json::from_str(
            r#"{
              "voxel": { "backend": "cpu", "size": 0.25, "opacityThreshold": 0.05 },
              "mesh": { "mode": "faces" },
              "navmesh": { "enabled": false, "agentHeight": 1.6, "agentRadius": 0.2, "maxSlopeDegrees": 45.0, "cellSize": 0.1, "cellHeight": 0.05, "walkableClimb": 0.25, "minRegionSize": 4, "mergeRegionSize": 12 }
            }"#,
        )
        .unwrap();

        assert_eq!(config.reconstruction.method, ReconstructionMethod::Voxel);
        assert_eq!(config.reconstruction.adapter_command, None);
        assert_eq!(config.reconstruction.target_triangles, 50_000);
        assert_eq!(config.reconstruction.timeout_seconds, 900);
    }

    #[test]
    fn process_config_parses_all_reconstruction_methods() {
        for (value, method) in [
            ("voxel", ReconstructionMethod::Voxel),
            ("sugar", ReconstructionMethod::Sugar),
            ("poisson", ReconstructionMethod::Poisson),
        ] {
            let config: ProcessConfig = serde_json::from_str(&format!(
                r#"{{
                  "reconstruction": {{
                    "method": "{value}",
                    "adapterCommand": "adapter",
                    "targetTriangles": 1234,
                    "timeoutSeconds": 12
                  }}
                }}"#,
            ))
            .unwrap();

            assert_eq!(config.reconstruction.method, method);
            assert_eq!(
                config.reconstruction.adapter_command.as_deref(),
                Some("adapter")
            );
            assert_eq!(config.reconstruction.target_triangles, 1234);
            assert_eq!(config.reconstruction.timeout_seconds, 12);
        }
    }

    #[test]
    fn recipe_bundle_accepts_gui_camel_case_filter_cluster_fields() {
        let recipe: RecipeBundle = serde_json::from_str(
            r#"{
              "editRecipe": {
                "operations": [
                  {
                    "type": "filterCluster",
                    "coarseVoxelSize": 0.1,
                    "opacityThreshold": 0.1,
                    "seedPos": [0.0, 1.0, 0.0],
                    "minContribution": 0.1
                  }
                ]
              }
            }"#,
        )
        .unwrap();

        let operation = recipe.edit_recipe.unwrap().operations.remove(0);
        match operation {
            EditOperation::FilterCluster {
                coarse_voxel_size,
                opacity_threshold,
                seed_pos,
                min_contribution,
            } => {
                assert_eq!(coarse_voxel_size, 0.1);
                assert_eq!(opacity_threshold, 0.1);
                assert_eq!(seed_pos, [0.0, 1.0, 0.0]);
                assert_eq!(min_contribution, 0.1);
            }
            other => panic!("expected filter cluster operation, got {other:?}"),
        }
    }
}
