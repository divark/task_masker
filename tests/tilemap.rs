use cucumber::{given, then, when, World};
use futures::executor::block_on;

use std::path;
use std::path::{PathBuf, MAIN_SEPARATOR};
use tiled::{Loader, Map};

#[derive(Debug, PartialEq)]
pub struct MapGridDimensions {
    width: usize,
    height: usize,
}

impl MapGridDimensions {
    pub fn new(num_tiles_width: usize, num_tiles_height: usize) -> Self {
        Self {
            width: num_tiles_width,
            height: num_tiles_height,
        }
    }

    /// Returns the width in tiles.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Returns the height in tiles.
    pub fn height(&self) -> usize {
        self.height
    }
}

#[derive(Debug, PartialEq)]
pub struct TileDimensions {
    width: usize,
    height: usize,
}

impl TileDimensions {
    pub fn new(width: usize, height: usize) -> Self {
        Self { width, height }
    }

    /// Returns the height in pixels.
    pub fn height(&self) -> usize {
        self.height
    }
}

#[derive(Debug, PartialEq)]
pub struct TileGridCoordinates {
    x: usize,
    y: usize,
}

impl TileGridCoordinates {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    /// Returns the x grid coordinate.
    pub fn x(&self) -> usize {
        self.x
    }

    /// Returns the y grid coordinate.
    pub fn y(&self) -> usize {
        self.y
    }
}

#[derive(Debug, PartialEq)]
pub struct TilePixelCoordinates {
    px_x: usize,
    px_y: usize,
}

impl TilePixelCoordinates {
    pub fn new(px_x: usize, px_y: usize) -> Self {
        Self { px_x, px_y }
    }

    /// Sets the y coordinate.
    pub fn set_y(&mut self, desired_y: usize) {
        self.px_y = desired_y;
    }

    /// Gets the y coordinate.
    pub fn y(&self) -> usize {
        self.px_y
    }
}

#[derive(Debug, PartialEq)]
pub struct TileTexture {
    spritesheet_file: PathBuf,
    spritesheet_entry_idx: usize,
}

impl TileTexture {
    pub fn new(spritesheet_file: PathBuf, spritesheet_entry_idx: usize) -> Self {
        Self {
            spritesheet_file,
            spritesheet_entry_idx,
        }
    }
}

#[derive(Debug)]
pub struct Tile {
    dimensions: TileDimensions,
    pixel_pos: TilePixelCoordinates,
    grid_pos: TileGridCoordinates,

    texture: Option<TileTexture>,
}

impl Tile {
    /// Returns the dimensions (width and height) of this tile
    /// in a grid.
    pub fn get_tile_dimensions(&self) -> &TileDimensions {
        &self.dimensions
    }

    /// Returns the position of this tile in a grid.
    pub fn get_grid_coordinates(&self) -> &TileGridCoordinates {
        &self.grid_pos
    }

    /// Returns the pixel coordinates of this tile in a grid.
    pub fn get_pixel_coordinates(&self) -> &TilePixelCoordinates {
        &self.pixel_pos
    }

    /// Returns a mutable reference to the pixel coordinates of this tile.
    pub fn get_pixel_coordinates_mut(&mut self) -> &mut TilePixelCoordinates {
        &mut self.pixel_pos
    }

    /// Returns the Tile Texture of this tile if found, or None
    /// otherwise.
    pub fn get_tile_texture(&self) -> Option<&TileTexture> {
        self.texture.as_ref()
    }
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

    /// Returns a mutable reference to the currently loaded Tilemap.
    pub fn tilemap_mut(&mut self) -> &mut Tilemap {
        &mut self.tilemap
    }
}

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

/// Returns a Texture found for some Tile at the specified coordinates,
/// or None otherwise.
fn get_texture_from_tiled(
    tiled_map: &Map,
    grid_coordinates: &TileGridCoordinates,
) -> Option<TileTexture> {
    let tile_layer = tiled_map.get_layer(0).unwrap().as_tile_layer().unwrap();

    let tile_grid_x = grid_coordinates.x() as i32;
    let tile_grid_y = grid_coordinates.y() as i32;
    if let Some(tile) = tile_layer.get_tile(tile_grid_x, tile_grid_y) {
        let sprite_idx = tile.id() as usize;
        let spritesheet_file = tile
            .get_tileset()
            .image
            .clone()
            .unwrap()
            .source
            .canonicalize()
            .unwrap();
        // NOTE: Windows returns a weird path without this.
        let spritesheet_path_absolute = path::absolute(spritesheet_file.as_path()).unwrap();

        Some(TileTexture::new(spritesheet_path_absolute, sprite_idx))
    } else {
        None
    }
}

#[derive(Debug)]
pub struct Tilemap {
    map_grid_dimensions: MapGridDimensions,
    tiles: Vec<Tile>,
}

impl Tilemap {
    pub fn new() -> Self {
        Self {
            tiles: Vec::new(),
            map_grid_dimensions: MapGridDimensions::new(0, 0),
        }
    }

    /// Populates tiles found from some tiled map.
    pub fn load_tiles_from_tiled_map(&mut self, tiled_map_path: &PathBuf) {
        let mut tiles = Vec::new();

        let mut tiled_loader = Loader::new();
        let tiled_map = tiled_loader
            .load_tmx_map(tiled_map_path)
            .expect("load_tiles_from_tiled: Could not load Tiled map");

        let tile_width = tiled_map.tile_width as usize;
        let tile_height = tiled_map.tile_height as usize;

        let map_width = tiled_map.width as usize;
        let map_height = tiled_map.height as usize;
        let map_grid_dimensions = MapGridDimensions::new(map_width, map_height);

        for x in 0..map_width {
            for y in 0..map_height {
                let tile_grid_pos = TileGridCoordinates::new(x, y);
                let tile_texture = get_texture_from_tiled(&tiled_map, &tile_grid_pos);

                let tile = Tile {
                    dimensions: TileDimensions::new(tile_width, tile_height),
                    pixel_pos: TilePixelCoordinates::new(tile_width * x, tile_height * y),
                    grid_pos: tile_grid_pos,
                    texture: tile_texture,
                };

                tiles.push(tile);
            }
        }

        self.tiles = tiles;
        self.map_grid_dimensions = map_grid_dimensions;
    }

    /// Returns the tiles currently loaded.
    pub fn get_tiles(&self) -> &Vec<Tile> {
        &self.tiles
    }

    /// "Flips" the y-axis for all loaded tiles.
    pub fn flip_y_axis(&mut self) {
        for tile in &mut self.tiles {
            let tile_grid_coords = tile.get_grid_coordinates();

            let recalculated_y = self.map_grid_dimensions.height() - tile_grid_coords.y() - 1;
            let flipped_y_coordinate = tile.get_tile_dimensions().height() * recalculated_y;

            let tile_pixel_coordinates = tile.get_pixel_coordinates_mut();
            tile_pixel_coordinates.set_y(flipped_y_coordinate);
        }
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

#[then(regex = r"there should be (\d+) Tiles? loaded from the Tiled map.")]
fn check_tile_is_loaded(tiled_context: &mut TiledContext, num_tiles_expected: String) {
    let expected_num_tiles = num_tiles_expected.parse::<usize>().unwrap();
    let actual_tiles = tiled_context.get_tiles();

    assert_eq!(expected_num_tiles, actual_tiles.len());
}

#[then(regex = r"Tile (\d+) should have a width of (\d+), and a height of (\d+).")]
fn check_tile_has_correct_width_and_height(
    tiled_context: &mut TiledContext,
    tile_num: String,
    width_expected: String,
    height_expected: String,
) {
    let expected_width = width_expected.parse::<usize>().unwrap();
    let expected_height = height_expected.parse::<usize>().unwrap();
    let expected_dimensions = TileDimensions::new(expected_width, expected_height);

    let tile_idx = tile_num.parse::<usize>().unwrap() - 1;
    let actual_dimensions = tiled_context.get_tiles()[tile_idx].get_tile_dimensions();

    assert_eq!(expected_dimensions, *actual_dimensions);
}

#[then(regex = r"Tile (\d+) should be at grid coordinates (\d+), (\d+).")]
fn check_tile_in_correct_grid_coordinates(
    tiled_context: &mut TiledContext,
    tile_num: String,
    grid_pos_x: String,
    grid_pos_y: String,
) {
    let grid_x = grid_pos_x.parse::<usize>().unwrap();
    let grid_y = grid_pos_y.parse::<usize>().unwrap();
    let expected_tile_grid_coordinates = TileGridCoordinates::new(grid_x, grid_y);

    let tile_idx = tile_num.parse::<usize>().unwrap() - 1;
    let actual_tile_grid_coordinates = tiled_context.get_tiles()[tile_idx].get_grid_coordinates();

    assert_eq!(
        expected_tile_grid_coordinates,
        *actual_tile_grid_coordinates
    );
}

#[then(regex = r"Tile (\d+) should be at pixel coordinates (\d+), (\d+).")]
fn check_tile_in_correct_pixel_coordinates(
    tiled_context: &mut TiledContext,
    tile_num: String,
    expected_px_x: String,
    expected_px_y: String,
) {
    let px_x = expected_px_x.parse::<usize>().unwrap();
    let px_y = expected_px_y.parse::<usize>().unwrap();
    let expected_pixel_coordinates = TilePixelCoordinates::new(px_x, px_y);

    let tile_idx = tile_num.parse::<usize>().unwrap() - 1;
    let actual_pixel_coordinates = tiled_context.get_tiles()[tile_idx].get_pixel_coordinates();

    assert_eq!(expected_pixel_coordinates, *actual_pixel_coordinates);
}

#[then(regex = r"Tile (\d+) should have a Texture pointing to entry (\d+) in (.+\.png).")]
fn check_one_tile_has_correct_texture_file(
    tiled_context: &mut TiledContext,
    tile_num: String,
    texture_entry: String,
    texture_filename: String,
) {
    let spritesheet_relative_path = format!("environment{}{}", MAIN_SEPARATOR, texture_filename);
    let expected_spritesheet_file = get_test_asset_path(&spritesheet_relative_path);
    let expected_tile_texture_entry = texture_entry.parse::<usize>().unwrap();
    let expected_tile_texture =
        TileTexture::new(expected_spritesheet_file, expected_tile_texture_entry);

    let tile_idx = tile_num.parse::<usize>().unwrap() - 1;
    let actual_tile_texture = tiled_context.get_tiles()[tile_idx]
        .get_tile_texture()
        .unwrap();

    assert_eq!(expected_tile_texture, *actual_tile_texture);
}

fn main() {
    block_on(TiledContext::run("tests/feature-files/tilemap.feature"));
}
