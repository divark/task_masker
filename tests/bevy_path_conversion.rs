use std::path::PathBuf;

pub fn to_bevy_path(input_path: PathBuf) -> PathBuf {
    let mut new_path = PathBuf::new();

    let mut path_element_stack = Vec::new();
    for path_element in input_path.iter().rev() {
        if path_element == "assets" {
            break;
        }

        path_element_stack.push(path_element);
    }

    while let Some(path_element) = path_element_stack.pop() {
        new_path.push(path_element);
    }

    new_path
}

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
