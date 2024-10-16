use bevy::prelude::*;
use bevy::render::settings::{RenderCreation, WgpuSettings};
use bevy::render::RenderPlugin;
use bevy::sprite::SpritePlugin;

use cucumber::{given, then, when, World};
use futures::executor::block_on;

use std::path::{PathBuf, MAIN_SEPARATOR};
use task_masker::map::path_finding::*;
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

/// Returns a Bevy App(lication) suitable for testing from a game context,
/// even without the presence of a display.
fn create_testable_bevy_app() -> App {
    let mut app = App::new();

    app.add_plugins(MinimalPlugins);
    app.add_plugins(WindowPlugin::default());
    app.add_plugins(AssetPlugin::default());
    app.add_plugins(RenderPlugin {
        render_creation: RenderCreation::from(WgpuSettings {
            backends: None,
            ..default()
        }),
        ..default()
    });
    app.add_plugins(SpritePlugin);
    app.add_plugins(ImagePlugin::default());

    app.update();

    app
}

#[derive(Debug, World)]
#[world(init = Self::new)]
pub struct TiledContext {
    testing_app: App,

    map_file_path: PathBuf,
    tilemap: Option<Tilemap>,
    tile_heightmap: Vec<usize>,
}

impl Default for TiledContext {
    fn default() -> Self {
        Self::new()
    }
}

impl TiledContext {
    pub fn new() -> Self {
        Self {
            testing_app: create_testable_bevy_app(),

            map_file_path: PathBuf::new(),
            tilemap: Some(Tilemap::new()),
            tile_heightmap: Vec::new(),
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
        self.tilemap_mut().load_tiles_from_tiled_map(tiled_map_path);
    }

    /// Returns the tiles currently recorded.
    pub fn get_tiles(&self) -> &Vec<Tile> {
        self.tilemap().get_tiles()
    }

    /// Returns a Tile specified at the Grid Coordinate if found,
    /// or returns None otherwise.
    pub fn get_tile(&self, grid_coordinate: &TileGridCoordinates) -> Option<&Tile> {
        let tilemap_dimensions = self.tilemap().get_dimensions();
        let tilemap_width = tilemap_dimensions.width();
        let tilemap_height = tilemap_dimensions.height();
        let tilemap_area = tilemap_width * tilemap_height;

        let tile_idx = (grid_coordinate.z() * tilemap_area)
            + (grid_coordinate.x() * tilemap_width)
            + grid_coordinate.y();
        self.tilemap().get_tiles().get(tile_idx)
    }

    /// Returns a mutable reference to the currently loaded Tilemap.
    pub fn tilemap_mut(&mut self) -> &mut Tilemap {
        self.tilemap.as_mut().unwrap()
    }

    /// Returns a reference to the currently loaded Tilemap.
    pub fn tilemap(&self) -> &Tilemap {
        self.tilemap.as_ref().unwrap()
    }

    /// Consumes the tilemap loaded.
    pub fn take_tilemap(&mut self) -> Tilemap {
        self.tilemap.take().unwrap()
    }

    /// Returns an instance of the Asset Server from the loaded Bevy App.
    pub fn get_asset_server(&self) -> AssetServer {
        self.testing_app.world().resource::<AssetServer>().clone()
    }

    /// Returns the collection of Texture Atlas Assets recorded from the loaded Bevy App.
    pub fn get_texture_atlas_assets_mut(&mut self) -> Mut<Assets<TextureAtlasLayout>> {
        self.testing_app
            .world_mut()
            .resource_mut::<Assets<TextureAtlasLayout>>()
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

#[when("the tile coordinates have been y-sorted,")]
fn y_sort_tile_coordinates(tiled_context: &mut TiledContext) {
    tiled_context.tilemap_mut().y_sort_tiles();
}

#[when("the tiles have been converted into a height map,")]
fn convert_tiles_into_heightmap(tiled_context: &mut TiledContext) {
    let tile_grid_coordinates = tiled_context
        .get_tiles()
        .iter()
        .map(|tile| tile.get_grid_coordinates())
        .collect::<Vec<&TileGridCoordinates>>();
    tiled_context.tile_heightmap = height_map_from(&tile_grid_coordinates);
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
    regex = r"Tile (\d+), (\d+), (\d+) should have a Texture using the spritesheet file (.+\.png)."
)]
fn check_one_tile_has_correct_texture_file(
    tiled_context: &mut TiledContext,
    tile_x: String,
    tile_y: String,
    tile_z: String,
    texture_filename: String,
) {
    let spritesheet_relative_path = format!("environment{}{}", MAIN_SEPARATOR, texture_filename);
    let expected_tile_texture_path = get_test_asset_path(&spritesheet_relative_path);

    let tile_grid_x = tile_x.parse::<usize>().unwrap();
    let tile_grid_y = tile_y.parse::<usize>().unwrap();
    let tile_grid_z = tile_z.parse::<usize>().unwrap();
    let tile_grid_coordinate = TileGridCoordinates::new_3d(tile_grid_x, tile_grid_y, tile_grid_z);

    let actual_tile_texture_path = tiled_context
        .get_tile(&tile_grid_coordinate)
        .unwrap()
        .get_tile_texture()
        .unwrap()
        .get_sprite()
        .get_path();
    assert_eq!(expected_tile_texture_path, *actual_tile_texture_path);
}

#[then(regex = r"Tile (\d+), (\d+), (\d+) should have a Texture pointing to sprite entry (\d+).")]
fn check_one_tile_has_correct_texture_entry(
    tiled_context: &mut TiledContext,
    tile_x: String,
    tile_y: String,
    tile_z: String,
    texture_entry: String,
) {
    let expected_tile_texture_entry = texture_entry.parse::<usize>().unwrap();

    let tile_grid_x = tile_x.parse::<usize>().unwrap();
    let tile_grid_y = tile_y.parse::<usize>().unwrap();
    let tile_grid_z = tile_z.parse::<usize>().unwrap();
    let tile_grid_coordinate = TileGridCoordinates::new_3d(tile_grid_x, tile_grid_y, tile_grid_z);
    let actual_tile_texture_entry = tiled_context
        .get_tile(&tile_grid_coordinate)
        .unwrap()
        .get_tile_texture()
        .unwrap()
        .get_sprite()
        .get_index();

    assert_eq!(expected_tile_texture_entry, actual_tile_texture_entry);
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

#[then(regex = r"Tile (\d+), (\d+), (\d+) should have (\d+) rows in its' Texture.")]
fn check_tile_texture_has_n_rows(
    tiled_context: &mut TiledContext,
    tile_x: String,
    tile_y: String,
    tile_z: String,
    num_rows_expected: String,
) {
    let tile_grid_x = tile_x.parse::<usize>().unwrap();
    let tile_grid_y = tile_y.parse::<usize>().unwrap();
    let tile_grid_z = tile_z.parse::<usize>().unwrap();
    let tile_grid_coordinate = TileGridCoordinates::new_3d(tile_grid_x, tile_grid_y, tile_grid_z);
    let tile_texture = tiled_context
        .get_tile(&tile_grid_coordinate)
        .unwrap()
        .get_tile_texture()
        .unwrap();
    let texture_dimensions = tile_texture.get_spritesheet_dimensions();

    let expected_num_rows = num_rows_expected.parse::<usize>().unwrap();
    let actual_num_rows = texture_dimensions.rows();
    assert_eq!(expected_num_rows, actual_num_rows);
}

#[then(regex = r"Tile (\d+), (\d+), (\d+) should have (\d+) columns in its' Texture.")]
fn check_tile_texture_has_n_columns(
    tiled_context: &mut TiledContext,
    tile_x: String,
    tile_y: String,
    tile_z: String,
    num_columns_expected: String,
) {
    let tile_grid_x = tile_x.parse::<usize>().unwrap();
    let tile_grid_y = tile_y.parse::<usize>().unwrap();
    let tile_grid_z = tile_z.parse::<usize>().unwrap();
    let tile_grid_coordinate = TileGridCoordinates::new_3d(tile_grid_x, tile_grid_y, tile_grid_z);
    let tile_texture = tiled_context
        .get_tile(&tile_grid_coordinate)
        .unwrap()
        .get_tile_texture()
        .unwrap();
    let texture_dimensions = tile_texture.get_spritesheet_dimensions();

    let expected_num_columns = num_columns_expected.parse::<usize>().unwrap();
    let actual_num_columns = texture_dimensions.columns();
    assert_eq!(expected_num_columns, actual_num_columns);
}

#[then("the number of render tiles created should match the number of Tiles with a Texture.")]
fn check_render_tiles_equal_tiles_with_texture(tiled_context: &mut TiledContext) {
    let tilemap = tiled_context.take_tilemap();
    let asset_server = tiled_context.get_asset_server();
    let mut texture_atlas_assets = tiled_context.get_texture_atlas_assets_mut();
    let render_tiles =
        convert_tilemap_to_bevy_render_tiles(&tilemap, &asset_server, &mut texture_atlas_assets);

    let tiles = tilemap.get_tiles();
    let expected_num_render_tiles = tiles
        .iter()
        .filter(|tile| tile.get_tile_texture().is_some())
        .count();
    let actual_num_render_tiles = render_tiles.len();
    assert_eq!(expected_num_render_tiles, actual_num_render_tiles);
}

#[then(
    regex = r"Tile (\d+), (\d+), (\d+)'s Texture should have a width of (\d+), and a height of (\d+)."
)]
fn check_tile_texture_has_correct_width_and_height(
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
        .get_tile_texture()
        .unwrap()
        .get_sprite()
        .get_sprite_dimensions();

    assert_eq!(expected_dimensions, *actual_dimensions);
}

#[then(regex = r"Tile (\d+), (\d+), (\d+) should be higher than Tile (\d+), (\d+), (\d+).")]
fn check_tile_heights(
    tiled_context: &mut TiledContext,
    first_tile_x: String,
    first_tile_y: String,
    first_tile_z: String,
    second_tile_x: String,
    second_tile_y: String,
    second_tile_z: String,
) {
    let first_tile_x_coord = first_tile_x.parse::<usize>().unwrap();
    let first_tile_y_coord = first_tile_y.parse::<usize>().unwrap();
    let first_tile_z_coord = first_tile_z.parse::<usize>().unwrap();
    let first_tile_grid_coordinates =
        TileGridCoordinates::new_3d(first_tile_x_coord, first_tile_y_coord, first_tile_z_coord);

    let second_tile_x_coord = second_tile_x.parse::<usize>().unwrap();
    let second_tile_y_coord = second_tile_y.parse::<usize>().unwrap();
    let second_tile_z_coord = second_tile_z.parse::<usize>().unwrap();
    let second_tile_grid_coordinates = TileGridCoordinates::new_3d(
        second_tile_x_coord,
        second_tile_y_coord,
        second_tile_z_coord,
    );

    let first_tile = tiled_context
        .get_tile(&first_tile_grid_coordinates)
        .expect("check_tile_height: Could not find first tile.");
    let second_tile = tiled_context
        .get_tile(&second_tile_grid_coordinates)
        .expect("check_tile_height: Could not find second tile.");

    let first_tile_height = first_tile.get_pixel_coordinates().z();
    let second_tile_height = second_tile.get_pixel_coordinates().z();
    assert!(first_tile_height > second_tile_height);
}

fn main() {
    block_on(TiledContext::run("tests/feature-files/tilemap.feature"));
}
