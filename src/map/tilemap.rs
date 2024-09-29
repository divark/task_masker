use bevy::prelude::*;
use std::path::PathBuf;
use tiled::{Loader, Map};

#[derive(Debug, PartialEq)]
pub struct MapGridDimensions {
    width: usize,
    height: usize,
    depth: usize,
}

impl MapGridDimensions {
    pub fn new(num_tiles_width: usize, num_tiles_height: usize) -> Self {
        Self {
            width: num_tiles_width,
            height: num_tiles_height,
            depth: 1,
        }
    }

    pub fn new_3d(width: usize, height: usize, depth: usize) -> Self {
        Self {
            width,
            height,
            depth,
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

    /// Returns the width in pixels.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Returns the height in pixels.
    pub fn height(&self) -> usize {
        self.height
    }
}

#[derive(Debug, PartialEq)]
pub struct TileDrawingOffset {
    x: isize,
    y: isize,
}

impl TileDrawingOffset {
    pub fn new(x: isize, y: isize) -> Self {
        Self { x, y }
    }

    /// Returns the horizontal offset (x) in pixels.
    pub fn x(&self) -> isize {
        self.x
    }

    /// Returns the vertical offset (y) in pixels.
    pub fn y(&self) -> isize {
        self.y
    }
}

#[derive(Debug, PartialEq, Component, Clone)]
pub struct TileGridCoordinates {
    x: usize,
    y: usize,
    z: usize,
}

impl TileGridCoordinates {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y, z: 0 }
    }

    pub fn new_3d(x: usize, y: usize, z: usize) -> Self {
        Self { x, y, z }
    }

    /// Returns the x grid coordinate.
    pub fn x(&self) -> usize {
        self.x
    }

    /// Returns the y grid coordinate.
    pub fn y(&self) -> usize {
        self.y
    }

    /// Returns the z grid coordinate.
    pub fn z(&self) -> usize {
        self.z
    }
}

#[derive(Debug, PartialEq)]
pub struct TilePixelCoordinates {
    px_x: f32,
    px_y: f32,
    px_z: f32,
}

impl TilePixelCoordinates {
    pub fn new(px_x: isize, px_y: isize) -> Self {
        Self {
            px_x: px_x as f32,
            px_y: px_y as f32,
            px_z: 0.0,
        }
    }

    pub fn new_3d(px_x: isize, px_y: isize, px_z: isize) -> Self {
        Self {
            px_x: px_x as f32,
            px_y: px_y as f32,
            px_z: px_z as f32,
        }
    }

    /// Sets the x coordinate.
    pub fn set_x(&mut self, desired_x: f32) {
        self.px_x = desired_x;
    }

    /// Gets the x coordinate.
    pub fn x(&self) -> f32 {
        self.px_x
    }

    /// Sets the y coordinate.
    pub fn set_y(&mut self, desired_y: f32) {
        self.px_y = desired_y;
    }

    /// Gets the y coordinate.
    pub fn y(&self) -> f32 {
        self.px_y
    }

    /// Gets the z coordinate.
    pub fn z(&self) -> f32 {
        self.px_z
    }
}

#[derive(Debug, PartialEq)]
pub struct TileSpriteSheet {
    spritesheet_file: PathBuf,
    spritesheet_entry_idx: usize,
}

impl TileSpriteSheet {
    pub fn new(spritesheet_file: PathBuf, spritesheet_entry_idx: usize) -> Self {
        Self {
            spritesheet_file,
            spritesheet_entry_idx,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct TileSprite {
    spritesheet_file: PathBuf,
    spritesheet_entry_idx: usize,
}

impl TileSprite {
    pub fn new(spritesheet_file: PathBuf, spritesheet_entry_idx: usize) -> Self {
        Self {
            spritesheet_file,
            spritesheet_entry_idx,
        }
    }

    /// Returns the path to the spritesheet recorded.
    pub fn get_path(&self) -> &PathBuf {
        &self.spritesheet_file
    }

    /// Returns the index to the spritesheet recorded.
    pub fn get_index(&self) -> usize {
        self.spritesheet_entry_idx
    }
}

#[derive(Debug, PartialEq)]
pub struct SpriteSheetDimensions {
    num_rows: usize,
    num_columns: usize,
}

impl SpriteSheetDimensions {
    pub fn new(num_rows: usize, num_columns: usize) -> Self {
        Self {
            num_rows,
            num_columns,
        }
    }

    /// Returns the number of rows found in the recorded spritesheet.
    pub fn rows(&self) -> usize {
        self.num_rows
    }

    /// Returns the number of columns found in the recorded spritesheet.
    pub fn columns(&self) -> usize {
        self.num_columns
    }
}

#[derive(Debug, PartialEq)]
pub struct TileTexture {
    sprite: TileSprite,
    spritesheet_dimensions: SpriteSheetDimensions,
}

impl TileTexture {
    pub fn new(sprite: TileSprite, spritesheet_dimensions: SpriteSheetDimensions) -> Self {
        Self {
            sprite,
            spritesheet_dimensions,
        }
    }

    /// Returns a reference to the sprite recorded.
    pub fn sprite(&self) -> &TileSprite {
        &self.sprite
    }

    /// Returns a reference to the spritesheet dimensions.
    pub fn dimensions(&self) -> &SpriteSheetDimensions {
        &self.spritesheet_dimensions
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

/// Returns a Texture found for some Tile at the specified coordinates,
/// or None otherwise.
fn get_texture_from_tiled(
    tiled_map: &Map,
    grid_coordinates: &TileGridCoordinates,
) -> Option<TileTexture> {
    let tile_grid_z = grid_coordinates.z();
    let tile_layer = tiled_map
        .get_layer(tile_grid_z)
        .unwrap()
        .as_tile_layer()
        .unwrap();

    let tile_grid_x = grid_coordinates.x() as i32;
    let tile_grid_y = grid_coordinates.y() as i32;
    if let Some(tile) = tile_layer.get_tile(tile_grid_x, tile_grid_y) {
        let tile_tileset = tile.get_tileset();
        let spritesheet_image = tile_tileset.image.clone().unwrap();
        let spritesheet_file = spritesheet_image.source;
        // NOTE: Why not fs::canonicalize? Because on Windows only, this adds a weird
        // prefix at the beginning of the path, making the tests fail. This is called
        // a UNC path, and we remove these with dunce:
        // https://docs.rs/dunce/latest/dunce/
        let spritesheet_file_path = dunce::canonicalize(spritesheet_file).unwrap();
        let sprite_idx = tile.id() as usize;
        let tile_sprite = TileSprite::new(spritesheet_file_path, sprite_idx);

        let num_rows = (spritesheet_image.height as u32 / tile_tileset.tile_height) as usize;
        let num_columns = (spritesheet_image.width as u32 / tile_tileset.tile_width) as usize;
        let tile_spritesheet_dimensions = SpriteSheetDimensions::new(num_rows, num_columns);

        Some(TileTexture::new(tile_sprite, tile_spritesheet_dimensions))
    } else {
        None
    }
}

/// Returns Drawing Offsets represented in pixels if found from a Tile's
/// texture, or defaults to 0, 0 otherwise.
fn get_drawing_offsets_from_tiled(
    tiled_map: &Map,
    grid_coordinates: &TileGridCoordinates,
) -> TileDrawingOffset {
    let tile_grid_z = grid_coordinates.z();
    let tile_layer = tiled_map
        .get_layer(tile_grid_z)
        .unwrap()
        .as_tile_layer()
        .unwrap();

    let tile_grid_x = grid_coordinates.x() as i32;
    let tile_grid_y = grid_coordinates.y() as i32;

    if let Some(tile) = tile_layer.get_tile(tile_grid_x, tile_grid_y) {
        let tileset = tile.get_tileset();
        let tile_offset_x = tileset.offset_x as isize;
        let tile_offset_y = tileset.offset_y as isize;

        TileDrawingOffset::new(tile_offset_x, tile_offset_y)
    } else {
        TileDrawingOffset::new(0, 0)
    }
}

/// Returns a relative path pointing to some asset in
/// the assets folder, or returns the path as-is if the
/// assets folder is not found.
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
        let map_depth = tiled_map.layers().len();
        let map_grid_dimensions = MapGridDimensions::new_3d(map_width, map_height, map_depth);

        for z in 0..map_depth {
            // TODO: Consider horizontal layer offsets as well, and
            // making this into MapLayerOffset.
            let vertical_layer_offset = tiled_map.layers().nth(z).unwrap().offset_y;
            for x in 0..map_width {
                for y in 0..map_height {
                    let tile_grid_pos = TileGridCoordinates::new_3d(x, y, z);
                    let tile_texture = get_texture_from_tiled(&tiled_map, &tile_grid_pos);
                    let drawing_offsets =
                        get_drawing_offsets_from_tiled(&tiled_map, &tile_grid_pos);

                    let tile_px_x = drawing_offsets.x() + (tile_width * x) as isize;
                    let tile_px_y = drawing_offsets.y()
                        + vertical_layer_offset as isize
                        + (tile_height * y) as isize;
                    let tile_px_z = z as isize;
                    let tile = Tile {
                        dimensions: TileDimensions::new(tile_width, tile_height),
                        pixel_pos: TilePixelCoordinates::new_3d(tile_px_x, tile_px_y, tile_px_z),
                        grid_pos: tile_grid_pos,
                        texture: tile_texture,
                    };

                    tiles.push(tile);
                }
            }
        }

        self.tiles = tiles;
        self.map_grid_dimensions = map_grid_dimensions;
    }

    /// Returns the dimensions of the Tile map in Tiles.
    pub fn get_dimensions(&self) -> &MapGridDimensions {
        &self.map_grid_dimensions
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
            tile_pixel_coordinates.set_y(flipped_y_coordinate as f32);
        }
    }

    /// Converts the pixel coordinates of each Tile to the isometric coordinate system.
    pub fn to_isometric_coordinates(&mut self) {
        for tile in &mut self.tiles {
            let tile_px_x = tile.get_pixel_coordinates().x();
            let tile_px_y = tile.get_pixel_coordinates().y();

            // Used the following as reference:
            // https://code.tutsplus.com/creating-isometric-worlds-a-primer-for-game-developers--gamedev-6511t
            let isometric_px_x = tile_px_x - tile_px_y;
            let isometric_px_y = (tile_px_x + tile_px_y) / 2.0;

            let tile_pixel_coordinates = tile.get_pixel_coordinates_mut();
            tile_pixel_coordinates.set_x(isometric_px_x);
            tile_pixel_coordinates.set_y(isometric_px_y);
        }
    }
}

#[derive(Debug, Bundle)]
pub struct RenderTile {
    grid_coordinate: TileGridCoordinates,
    tile_sprite: SpriteBundle,
    tile_texture_atlas: TextureAtlas,
}

impl RenderTile {
    pub fn new(
        grid_coordinate: TileGridCoordinates,
        sprite: SpriteBundle,
        texture_atlas: TextureAtlas,
    ) -> Self {
        Self {
            grid_coordinate,
            tile_sprite: sprite,
            tile_texture_atlas: texture_atlas,
        }
    }
}

/// Returns a collection of Render Tiles converted from some Tilemap for the
/// Bevy game engine.
/// Precondition: Tilemap was called with a load method.
pub fn convert_tilemap_to_bevy_render_tiles(
    tilemap: &Tilemap,
    asset_server: AssetServer,
    mut texture_atlas_assets: Mut<Assets<TextureAtlasLayout>>,
) -> Vec<RenderTile> {
    let mut render_tiles = Vec::new();

    let tiles = tilemap.get_tiles();
    for tile in tiles {
        if tile.get_tile_texture().is_none() {
            continue;
        }

        let tile_dimensions = tile.get_tile_dimensions();
        let tile_texture = tile.get_tile_texture().unwrap();
        let tile_grid_coordinate = tile.get_grid_coordinates().to_owned();
        let tile_pixel_coordinates = tile.get_pixel_coordinates();
        let tile_sprite = tile_texture.sprite();
        let tile_spritesheet_dimensions = tile_texture.dimensions();

        let bevy_tile_texture = asset_server.load(to_bevy_path(tile_sprite.get_path().clone()));

        let tile_size = UVec2::new(
            tile_dimensions.width() as u32,
            tile_dimensions.height() as u32,
        );
        let tile_texture_layout = TextureAtlasLayout::from_grid(
            tile_size,
            tile_spritesheet_dimensions.columns() as u32,
            tile_spritesheet_dimensions.rows() as u32,
            None,
            None,
        );
        let tile_texture_atlas_layout = texture_atlas_assets.add(tile_texture_layout);

        let bevy_tile_sprite = SpriteBundle {
            transform: Transform::from_xyz(
                tile_pixel_coordinates.x(),
                tile_pixel_coordinates.y(),
                tile_pixel_coordinates.z(),
            ),
            texture: bevy_tile_texture,
            ..default()
        };

        let tile_texture_atlas = TextureAtlas {
            layout: tile_texture_atlas_layout,
            index: tile_sprite.get_index(),
        };

        let render_tile =
            RenderTile::new(tile_grid_coordinate, bevy_tile_sprite, tile_texture_atlas);
        render_tiles.push(render_tile);
    }

    render_tiles
}
