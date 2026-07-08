use crate::paths::expand_user_path;
use augmented_gaussian_core::readers::{read_ply, read_ply_metadata, write_ply};
use augmented_gaussian_core::splat_table::SplatTable;
use serde::Serialize;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

#[cfg(not(test))]
const PREVIEW_SPLAT_LIMIT: usize = 5_000_000;
#[cfg(test)]
const PREVIEW_SPLAT_LIMIT: usize = 1_000;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceMetadata {
    pub path: String,
    pub bytes: u64,
    pub format: String,
    pub splat_count: usize,
    pub bounds: Option<augmented_gaussian_core::math::Bounds>,
    pub preview_path: Option<String>,
    pub preview_splat_count: Option<usize>,
}

pub fn load_source(path: String) -> Result<SourceMetadata, String> {
    let resolved_path = expand_user_path(PathBuf::from(&path))?;
    let absolute_path = std::fs::canonicalize(&resolved_path)
        .map_err(|err| format!("failed to resolve absolute path for '{}': {}", path, err))?;
    let file_metadata = std::fs::metadata(&absolute_path).map_err(|err| err.to_string())?;
    let ext = absolute_path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let (format, splat_count, preview_path, preview_splat_count) = match ext.as_str() {
        "ply" => {
            let metadata = read_ply_metadata(&absolute_path).map_err(|err| err.to_string())?;
            let preview = if metadata.vertex_count > PREVIEW_SPLAT_LIMIT {
                Some(load_or_write_preview_ply(
                    &absolute_path,
                    file_metadata.len(),
                    metadata.vertex_count,
                )?)
            } else {
                None
            };
            (
                "ply".to_string(),
                metadata.vertex_count,
                preview
                    .as_ref()
                    .map(|(path, _)| path.to_string_lossy().to_string()),
                preview.map(|(_, count)| count),
            )
        }
        "splat" => {
            if file_metadata.len() == 0 || file_metadata.len() % 32 != 0 {
                return Err(format!(
                    "invalid .splat byte length {}; expected a non-empty multiple of 32",
                    file_metadata.len()
                ));
            }
            (
                "splat".to_string(),
                (file_metadata.len() / 32) as usize,
                None,
                None,
            )
        }
        _ => return Err(format!("unsupported input format: {ext}")),
    };

    Ok(SourceMetadata {
        path: absolute_path.to_string_lossy().to_string(),
        bytes: file_metadata.len(),
        format,
        splat_count,
        bounds: None,
        preview_path,
        preview_splat_count,
    })
}

fn load_or_write_preview_ply(
    source_path: &PathBuf,
    source_bytes: u64,
    source_count: usize,
) -> Result<(PathBuf, usize), String> {
    let preview_count = source_count.min(PREVIEW_SPLAT_LIMIT);
    let preview_path =
        preview_path_for_source(source_path, source_bytes, source_count, preview_count);
    if cached_preview_matches(&preview_path, preview_count) {
        return Ok((preview_path, preview_count));
    }

    write_preview_ply(source_path, preview_path, preview_count)
}

fn cached_preview_matches(preview_path: &PathBuf, preview_count: usize) -> bool {
    read_ply_metadata(preview_path)
        .map(|metadata| metadata.vertex_count == preview_count)
        .unwrap_or(false)
}

fn write_preview_ply(
    source_path: &PathBuf,
    preview_path: PathBuf,
    preview_count: usize,
) -> Result<(PathBuf, usize), String> {
    let source = read_ply(source_path).map_err(|err| err.to_string())?;
    let preview = downsample_table(&source, preview_count);
    write_ply_atomically(&preview_path, &preview)?;
    Ok((preview_path, preview.len()))
}

fn write_ply_atomically(preview_path: &PathBuf, preview: &SplatTable) -> Result<(), String> {
    let temp_path = temp_preview_path(preview_path);
    write_ply(&temp_path, preview).map_err(|err| err.to_string())?;
    match std::fs::rename(&temp_path, preview_path) {
        Ok(()) => Ok(()),
        Err(first_err) => {
            let _ = std::fs::remove_file(preview_path);
            std::fs::rename(&temp_path, preview_path).map_err(|second_err| {
                let _ = std::fs::remove_file(&temp_path);
                format!(
                    "failed to install preview cache '{}': {}; retry after removing target failed: {}",
                    preview_path.to_string_lossy(),
                    first_err,
                    second_err
                )
            })
        }
    }
}

fn temp_preview_path(preview_path: &PathBuf) -> PathBuf {
    let file_name = preview_path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("augmented-gaussian-preview.ply");
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    preview_path.with_file_name(format!("{file_name}.{}.{}.tmp", std::process::id(), unique))
}

fn preview_path_for_source(
    source_path: &PathBuf,
    source_bytes: u64,
    source_count: usize,
    preview_count: usize,
) -> PathBuf {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    source_path.hash(&mut hasher);
    source_bytes.hash(&mut hasher);
    source_count.hash(&mut hasher);
    preview_count.hash(&mut hasher);
    if let Ok(modified) = std::fs::metadata(source_path).and_then(|metadata| metadata.modified()) {
        modified.hash(&mut hasher);
    }
    std::env::temp_dir().join(format!(
        "augmented-gaussian-preview-{:016x}.ply",
        hasher.finish()
    ))
}

fn downsample_table(source: &SplatTable, preview_count: usize) -> SplatTable {
    let source_count = source.len();
    let mut preview = SplatTable {
        x: Vec::with_capacity(preview_count),
        y: Vec::with_capacity(preview_count),
        z: Vec::with_capacity(preview_count),
        scale_0: Vec::with_capacity(preview_count),
        scale_1: Vec::with_capacity(preview_count),
        scale_2: Vec::with_capacity(preview_count),
        opacity: Vec::with_capacity(preview_count),
        f_dc_0: Vec::with_capacity(preview_count),
        f_dc_1: Vec::with_capacity(preview_count),
        f_dc_2: Vec::with_capacity(preview_count),
        rot_0: Vec::with_capacity(preview_count),
        rot_1: Vec::with_capacity(preview_count),
        rot_2: Vec::with_capacity(preview_count),
        rot_3: Vec::with_capacity(preview_count),
        f_rest: source
            .f_rest
            .iter()
            .map(|_| Vec::with_capacity(preview_count))
            .collect(),
    };

    for i in 0..preview_count {
        let source_index = if preview_count == source_count {
            i
        } else {
            (i * source_count) / preview_count
        };
        preview.x.push(source.x[source_index]);
        preview.y.push(source.y[source_index]);
        preview.z.push(source.z[source_index]);
        preview.scale_0.push(source.scale_0[source_index]);
        preview.scale_1.push(source.scale_1[source_index]);
        preview.scale_2.push(source.scale_2[source_index]);
        preview.opacity.push(source.opacity[source_index]);
        preview.f_dc_0.push(source.f_dc_0[source_index]);
        preview.f_dc_1.push(source.f_dc_1[source_index]);
        preview.f_dc_2.push(source.f_dc_2[source_index]);
        preview.rot_0.push(source.rot_0[source_index]);
        preview.rot_1.push(source.rot_1[source_index]);
        preview.rot_2.push(source.rot_2[source_index]);
        preview.rot_3.push(source.rot_3[source_index]);
        for (preview_rest, source_rest) in preview.f_rest.iter_mut().zip(source.f_rest.iter()) {
            preview_rest.push(source_rest[source_index]);
        }
    }

    preview
}

#[cfg(test)]
mod tests {
    use super::*;
    use augmented_gaussian_core::readers::read_ply;
    use std::io::Write;

    fn minimal_ply_header(vertex_count: usize) -> Vec<u8> {
        format!(
            concat!(
                "ply\n",
                "format binary_little_endian 1.0\n",
                "element vertex {}\n",
                "property float x\n",
                "property float y\n",
                "property float z\n",
                "property float scale_0\n",
                "property float scale_1\n",
                "property float scale_2\n",
                "property float opacity\n",
                "property float f_dc_0\n",
                "property float f_dc_1\n",
                "property float f_dc_2\n",
                "property float rot_0\n",
                "property float rot_1\n",
                "property float rot_2\n",
                "property float rot_3\n",
                "end_header\n",
            ),
            vertex_count
        )
        .into_bytes()
    }

    fn minimal_ply_bytes(vertex_count: usize) -> Vec<u8> {
        let mut bytes = minimal_ply_header(vertex_count);
        for i in 0..vertex_count {
            for value in [
                i as f32, 2.0, 3.0, 0.0, 0.1, 0.2, 1.0, 0.3, 0.4, 0.5, 1.0, 0.0, 0.0, 0.0,
            ] {
                bytes.extend_from_slice(&value.to_le_bytes());
            }
        }
        bytes
    }

    #[test]
    fn ply_load_source_uses_fast_metadata_without_preview_file() {
        let mut input = tempfile::Builder::new().suffix(".ply").tempfile().unwrap();
        input.write_all(&minimal_ply_bytes(1)).unwrap();

        let metadata = load_source(input.path().to_string_lossy().to_string()).unwrap();

        assert!(std::path::Path::new(&metadata.path).is_absolute());
        assert_eq!(metadata.format, "ply");
        assert_eq!(metadata.splat_count, 1);
        assert!(metadata.bounds.is_none());
        assert!(metadata.preview_path.is_none());
        assert!(metadata.preview_splat_count.is_none());
    }

    #[test]
    fn large_ply_load_source_writes_downsampled_preview_file() {
        let mut input = tempfile::Builder::new().suffix(".ply").tempfile().unwrap();
        input
            .write_all(&minimal_ply_bytes(PREVIEW_SPLAT_LIMIT + 2))
            .unwrap();

        let metadata = load_source(input.path().to_string_lossy().to_string()).unwrap();
        let preview_path = metadata.preview_path.as_ref().unwrap();
        let preview = read_ply(preview_path).unwrap();

        assert_eq!(metadata.format, "ply");
        assert_eq!(metadata.splat_count, PREVIEW_SPLAT_LIMIT + 2);
        assert_eq!(metadata.preview_splat_count, Some(PREVIEW_SPLAT_LIMIT));
        assert_eq!(preview.len(), PREVIEW_SPLAT_LIMIT);
        assert_eq!(preview.x[0], 0.0);
        let midpoint = PREVIEW_SPLAT_LIMIT / 2;
        assert_eq!(preview.x[midpoint], (midpoint + 1) as f32);
    }

    #[test]
    fn large_ply_load_source_reuses_existing_preview_file_without_decoding_source_rows() {
        let source_count = PREVIEW_SPLAT_LIMIT + 2;
        let preview_count = PREVIEW_SPLAT_LIMIT;
        let mut input = tempfile::Builder::new().suffix(".ply").tempfile().unwrap();
        input.write_all(&minimal_ply_header(source_count)).unwrap();
        input.flush().unwrap();

        let source_path = std::fs::canonicalize(input.path()).unwrap();
        let source_bytes = std::fs::metadata(&source_path).unwrap().len();
        let preview_path =
            preview_path_for_source(&source_path, source_bytes, source_count, preview_count);
        std::fs::write(&preview_path, minimal_ply_header(preview_count)).unwrap();

        let metadata = load_source(source_path.to_string_lossy().to_string()).unwrap();
        let expected_preview_path = preview_path.to_string_lossy().to_string();

        assert_eq!(metadata.format, "ply");
        assert_eq!(metadata.splat_count, source_count);
        assert_eq!(metadata.preview_splat_count, Some(preview_count));
        assert_eq!(
            metadata.preview_path.as_deref(),
            Some(expected_preview_path.as_str())
        );

        let _ = std::fs::remove_file(preview_path);
    }

    #[test]
    fn splat_load_source_derives_count_from_byte_length() {
        let mut input = tempfile::Builder::new()
            .suffix(".splat")
            .tempfile()
            .unwrap();
        input.write_all(&[0; 64]).unwrap();

        let metadata = load_source(input.path().to_string_lossy().to_string()).unwrap();

        assert!(std::path::Path::new(&metadata.path).is_absolute());
        assert_eq!(metadata.format, "splat");
        assert_eq!(metadata.splat_count, 2);
        assert!(metadata.bounds.is_none());
        assert!(metadata.preview_path.is_none());
        assert!(metadata.preview_splat_count.is_none());
    }
}
