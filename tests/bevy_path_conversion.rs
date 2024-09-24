use std::path::PathBuf;
use task_masker::map::tilemap::to_bevy_path;

#[test]
fn path_with_assets_trimmed() {
    // Given
    let input_path =
        PathBuf::from("/Users/divark/repos/task_masker/assets/environments/maps/main.tmx");
    // When
    let actual_path = to_bevy_path(input_path);

    // Then
    let expected_path = PathBuf::from("environments/maps/main.tmx");
    assert_eq!(expected_path, actual_path);
}

#[test]
fn path_without_assets_not_trimmed() {
    // Given
    let input_path =
        PathBuf::from("/Users/divark/repos/task_masker/tests/test-assets/environments/map.tmx");
    // When
    let actual_path = to_bevy_path(input_path.clone());

    // Then
    let expected_path = input_path.clone();
    assert_eq!(expected_path, actual_path);
}
