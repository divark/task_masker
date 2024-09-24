use cucumber::{given, then, when, World};
use futures::executor::block_on;

use std::path::{PathBuf, MAIN_SEPARATOR};
use task_masker::map::tilemap::*;

/// Returns a path to some asset requested from the test-assets
/// folder.
fn get_test_asset_path(desired_map_asset: &str) -> PathBuf {
    let mut map_path = PathBuf::new();

    if let Ok(project_root_directory) = std::env::var("CARGO_MANIFEST_DIR") {
        map_path.push(project_root_directory);
    }

    map_path.push("tests");
    map_path.push("test-assets");
    map_path.push(desired_map_asset);

    map_path
}

#[derive(Debug, World)]
#[world(init = Self::new)]
pub struct TiledContext {
    map_file_path: PathBuf,
    tilemap: Tilemap,
}

impl TiledContext {
    pub fn new() -> Self {
        Self {
            map_file_path: PathBuf::new(),
            tilemap: Tilemap::new(),
        }
    }

    /// Records the path of the map to be loaded.
    pub fn set_map_path(&mut self, map_path: PathBuf) {
        self.map_file_path = map_path;
    }

    /// Returns the path of the map to be loaded.
    pub fn get_map_path(&self) -> &PathBuf {
        &self.map_file_path
    }

    /// Loads a Tilemap from some Tiled map file.
    pub fn load_tiled_map(&mut self, tiled_map_path: &PathBuf) {
        self.tilemap.load_tiles_from_tiled_map(tiled_map_path);
    }

    /// Returns the tiles currently recorded.
    pub fn get_tiles(&self) -> &Vec<Tile> {
        self.tilemap.get_tiles()
    }

    /// Returns a Tile specified at the Grid Coordinate if found,
    /// or returns None otherwise.
    pub fn get_tile(&self, grid_coordinate: &TileGridCoordinates) -> Option<&Tile> {
        let tilemap_dimensions = self.tilemap.get_dimensions();
        let tilemap_width = tilemap_dimensions.width();
        let tilemap_height = tilemap_dimensions.height();
        let tilemap_area = tilemap_width * tilemap_height;

        let tile_idx = (grid_coordinate.z() * tilemap_area)
            + (grid_coordinate.y() * tilemap_width)
            + grid_coordinate.x();
        self.tilemap.get_tiles().get(tile_idx)
    }

    /// Returns a mutable reference to the currently loaded Tilemap.
    pub fn tilemap_mut(&mut self) -> &mut Tilemap {
        &mut self.tilemap
    }

    /// Returns a reference to the currently loaded Tilemap.
    pub fn tilemap(&self) -> &Tilemap {
        &self.tilemap
    }
}

#[given(regex = r"a Tiled map called (.+\.tmx),")]
fn given_tiled_map_exists(tiled_context: &mut TiledContext, tiled_map_name: String) {
    let tiled_map_relative_path = format!("maps/{}", tiled_map_name);
    let tiled_map_path = get_test_asset_path(&tiled_map_relative_path);
    assert!(tiled_map_path.exists());

    tiled_context.set_map_path(tiled_map_path);
}

#[when("the tiles are loaded from the Tiled map,")]
fn load_tiles_from_tiled_map(tiled_context: &mut TiledContext) {
    tiled_context.load_tiled_map(&tiled_context.get_map_path().clone());
}

#[when("the y-axis has been inverted for all tiles,")]
fn invert_y_axis_for_all_tiles(tiled_context: &mut TiledContext) {
    tiled_context.tilemap_mut().flip_y_axis();
}

#[when("the tile coordinates have been converted to isometric,")]
fn convert_tile_coordinates_to_isometric(tiled_context: &mut TiledContext) {
    tiled_context.tilemap_mut().to_isometric_coordinates();
}

#[then(regex = r"there should be (\d+) Tiles? loaded from the Tiled map.")]
fn check_tile_is_loaded(tiled_context: &mut TiledContext, num_tiles_expected: String) {
    let expected_num_tiles = num_tiles_expected.parse::<usize>().unwrap();
    let actual_tiles = tiled_context.get_tiles();

    assert_eq!(expected_num_tiles, actual_tiles.len());
}

#[then(regex = r"Tile (\d+), (\d+), (\d+) should have a width of (\d+), and a height of (\d+).")]
fn check_tile_has_correct_width_and_height(
    tiled_context: &mut TiledContext,
    tile_x: String,
    tile_y: String,
    tile_z: String,
    width_expected: String,
    height_expected: String,
) {
    let expected_width = width_expected.parse::<usize>().unwrap();
    let expected_height = height_expected.parse::<usize>().unwrap();
    let expected_dimensions = TileDimensions::new(expected_width, expected_height);

    let tile_grid_x = tile_x.parse::<usize>().unwrap();
    let tile_grid_y = tile_y.parse::<usize>().unwrap();
    let tile_grid_z = tile_z.parse::<usize>().unwrap();
    let tile_grid_coordinate = TileGridCoordinates::new_3d(tile_grid_x, tile_grid_y, tile_grid_z);
    let actual_dimensions = tiled_context
        .get_tile(&tile_grid_coordinate)
        .unwrap()
        .get_tile_dimensions();

    assert_eq!(expected_dimensions, *actual_dimensions);
}

#[then(regex = r"Tile (\d+), (\d+), (\d+) should be at grid coordinates (\d+), (\d+), (\d+).")]
fn check_tile_in_correct_grid_coordinates(
    tiled_context: &mut TiledContext,
    tile_x: String,
    tile_y: String,
    tile_z: String,
    grid_pos_x: String,
    grid_pos_y: String,
    grid_pos_z: String,
) {
    let grid_x = grid_pos_x.parse::<usize>().unwrap();
    let grid_y = grid_pos_y.parse::<usize>().unwrap();
    let grid_z = grid_pos_z.parse::<usize>().unwrap();
    let expected_tile_grid_coordinates = TileGridCoordinates::new_3d(grid_x, grid_y, grid_z);

    let tile_grid_x = tile_x.parse::<usize>().unwrap();
    let tile_grid_y = tile_y.parse::<usize>().unwrap();
    let tile_grid_z = tile_z.parse::<usize>().unwrap();
    let tile_grid_coordinate = TileGridCoordinates::new_3d(tile_grid_x, tile_grid_y, tile_grid_z);
    let actual_tile_grid_coordinates = tiled_context
        .get_tile(&tile_grid_coordinate)
        .unwrap()
        .get_grid_coordinates();

    assert_eq!(
        expected_tile_grid_coordinates,
        *actual_tile_grid_coordinates
    );
}

#[then(
    regex = r"Tile (\d+), (\d+), (\d+) should be at pixel coordinates (-?\d+), (-?\d+), (-?\d+)."
)]
fn check_tile_in_correct_pixel_coordinates(
    tiled_context: &mut TiledContext,
    tile_x: String,
    tile_y: String,
    tile_z: String,
    expected_px_x: String,
    expected_px_y: String,
    expected_px_z: String,
) {
    let px_x = expected_px_x.parse::<isize>().unwrap();
    let px_y = expected_px_y.parse::<isize>().unwrap();
    let px_z = expected_px_z.parse::<isize>().unwrap();
    let expected_pixel_coordinates = TilePixelCoordinates::new_3d(px_x, px_y, px_z);

    let tile_grid_x = tile_x.parse::<usize>().unwrap();
    let tile_grid_y = tile_y.parse::<usize>().unwrap();
    let tile_grid_z = tile_z.parse::<usize>().unwrap();
    let tile_grid_coordinate = TileGridCoordinates::new_3d(tile_grid_x, tile_grid_y, tile_grid_z);
    let actual_pixel_coordinates = tiled_context
        .get_tile(&tile_grid_coordinate)
        .unwrap()
        .get_pixel_coordinates();

    assert_eq!(expected_pixel_coordinates, *actual_pixel_coordinates);
}

#[then(
    regex = r"Tile (\d+), (\d+), (\d+) should have a Texture pointing to entry (\d+) in (.+\.png)."
)]
fn check_one_tile_has_correct_texture_file(
    tiled_context: &mut TiledContext,
    tile_x: String,
    tile_y: String,
    tile_z: String,
    texture_entry: String,
    texture_filename: String,
) {
    let spritesheet_relative_path = format!("environment{}{}", MAIN_SEPARATOR, texture_filename);
    let expected_spritesheet_file = get_test_asset_path(&spritesheet_relative_path);
    let expected_tile_texture_entry = texture_entry.parse::<usize>().unwrap();
    let expected_tile_texture =
        TileTexture::new(expected_spritesheet_file, expected_tile_texture_entry);

    let tile_grid_x = tile_x.parse::<usize>().unwrap();
    let tile_grid_y = tile_y.parse::<usize>().unwrap();
    let tile_grid_z = tile_z.parse::<usize>().unwrap();
    let tile_grid_coordinate = TileGridCoordinates::new_3d(tile_grid_x, tile_grid_y, tile_grid_z);
    let actual_tile_texture = tiled_context
        .get_tile(&tile_grid_coordinate)
        .unwrap()
        .get_tile_texture()
        .unwrap();

    assert_eq!(expected_tile_texture, *actual_tile_texture);
}

#[then(regex = r"Tile (\d+), (\d+), (\d+) should not have a Texture.")]
fn check_tile_has_no_texture(
    tiled_context: &mut TiledContext,
    tile_x: String,
    tile_y: String,
    tile_z: String,
) {
    let tile_grid_x = tile_x.parse::<usize>().unwrap();
    let tile_grid_y = tile_y.parse::<usize>().unwrap();
    let tile_grid_z = tile_z.parse::<usize>().unwrap();
    let tile_grid_coordinate = TileGridCoordinates::new_3d(tile_grid_x, tile_grid_y, tile_grid_z);
    let actual_tile_texture = tiled_context
        .get_tile(&tile_grid_coordinate)
        .unwrap()
        .get_tile_texture();

    assert!(actual_tile_texture.is_none());
}

#[then("the number of render tiles created should match the number of Tiles with a Texture.")]
fn check_render_tiles_equal_tiles_with_texture(tiled_context: &mut TiledContext) {
    let tiles = tiled_context.get_tiles();
    // TODO: Address notes in convert_tilemap_to_bevy_render_tiles before uncommenting this line.
    // Also, get a way to retrieve the asset_server and texture_atlas_assets from TiledContext.
    let render_tiles: Vec<RenderTile> = Vec::new(); //convert_tilemap_to_bevy_render_tiles(tiled_context.tilemap(), asset_server, texture_atlas_assets);

    let expected_num_render_tiles = tiles
        .iter()
        .filter(|tile| tile.get_tile_texture().is_some())
        .count();
    let actual_num_render_tiles = render_tiles.len();
    assert_eq!(expected_num_render_tiles, actual_num_render_tiles);
}

fn main() {
    block_on(TiledContext::run("tests/feature-files/tilemap.feature"));
}
