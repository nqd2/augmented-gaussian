use augmented_gaussian_core::output_dir::resolve_output_dir;
use std::fs;
use std::path::PathBuf;

#[test]
fn empty_output_dir_uses_default_root_with_timestamped_input_name() {
    let input = PathBuf::from("C:/scan/room.ply");
    let resolved = resolve_output_dir(&input, "", 123_456).unwrap();

    assert!(resolved.ends_with("Downloads/augmented-gaussian/room_123456"));
}

#[test]
fn default_export_root_gets_timestamped_child() {
    let input = PathBuf::from("room.splat");
    let resolved = resolve_output_dir(&input, "~/Downloads/augmented-gaussian", 123_456).unwrap();

    assert!(resolved.ends_with("Downloads/augmented-gaussian/room_123456"));
}

#[test]
fn explicit_child_output_dir_is_preserved() {
    let input = PathBuf::from("room.splat");
    let resolved = resolve_output_dir(&input, "~/Downloads/custom/foo", 123_456).unwrap();

    assert!(resolved.ends_with("Downloads/custom/foo"));
}

#[test]
fn generated_output_dir_uses_numeric_suffix_when_base_exists() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("Downloads").join("augmented-gaussian");
    fs::create_dir_all(root.join("room_123456")).unwrap();
    let input = PathBuf::from("room.ply");
    let resolved = resolve_output_dir(&input, &root, 123_456).unwrap();

    assert_eq!(resolved, root.join("room_123456_1"));
}
