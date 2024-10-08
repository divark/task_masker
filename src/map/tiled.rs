// How to use this:
//   You should copy/paste this into your project and use it much like examples/tiles.rs uses this
//   file. When you do so you will need to adjust the code based on whether you're using the
//   'atlas` feature in bevy_ecs_tilemap. The bevy_ecs_tilemap uses this as an example of how to
//   use both single image tilesets and image collection tilesets. Since your project won't have
//   the 'atlas' feature defined in your Cargo config, the expressions prefixed by the #[cfg(...)]
//   macro will not compile in your project as-is. If your project depends on the bevy_ecs_tilemap
//   'atlas' feature then move all of the expressions prefixed by #[cfg(not(feature = "atlas"))].
//   Otherwise remove all of the expressions prefixed by #[cfg(feature = "atlas")].
//
// Functional limitations:
//   * When the 'atlas' feature is enabled tilesets using a collection of images will be skipped.
//   * Only finite tile layers are loaded. Infinite tile layers and object layers will be skipped.
use tiled::Loader;

use std::io::{Cursor, ErrorKind};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use bevy::{
    asset::{io::Reader, AssetLoader, AssetPath, AsyncReadExt},
    log,
    prelude::*,
    reflect::TypePath,
    utils::HashMap,
};
use bevy_ecs_tilemap::prelude::*;

use thiserror::Error;

pub fn spawn_map(mut commands: Commands, asset_server: Res<AssetServer>) {
    let map_handle: Handle<TiledMap> = asset_server.load("TM_map.tmx");

    commands.spawn(TiledMapBundle {
        tiled_map: map_handle,
        render_settings: TilemapRenderSettings {
            render_chunk_size: UVec2::new(1280, 1),
            y_sort: true,
        },
        ..Default::default()
    });
}

#[derive(Default)]
pub struct TiledMapPlugin;

impl Plugin for TiledMapPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_asset::<TiledMap>()
            .register_asset_loader(TiledLoader)
            .add_systems(Update, process_loaded_maps);
    }
}

#[derive(TypePath, Asset)]
pub struct TiledMap {
    pub map: tiled::Map,

    pub tilemap_textures: HashMap<usize, TilemapTexture>,

    // The offset into the tileset_images for each tile id within each tileset.
    pub tile_image_offsets: HashMap<(usize, tiled::TileId), u32>,
}

// Stores a list of tiled layers.
#[derive(Component, Default)]
pub struct TiledLayersStorage {
    pub storage: HashMap<u32, Entity>,
}

// Stores the Layer number for some associated Tile.
#[derive(Component, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct LayerNumber(pub usize);

#[derive(Default, Bundle)]
pub struct TiledMapBundle {
    pub tiled_map: Handle<TiledMap>,
    pub storage: TiledLayersStorage,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub render_settings: TilemapRenderSettings,
}

struct BytesResourceReader {
    bytes: Arc<[u8]>,
}

impl BytesResourceReader {
    fn new(bytes: &[u8]) -> Self {
        Self {
            bytes: Arc::from(bytes),
        }
    }
}

impl tiled::ResourceReader for BytesResourceReader {
    type Resource = Cursor<Arc<[u8]>>;
    type Error = std::io::Error;

    fn read_from(&mut self, _path: &Path) -> std::result::Result<Self::Resource, Self::Error> {
        // In this case, the path is ignored because the byte data is already provided.
        Ok(Cursor::new(self.bytes.clone()))
    }
}

pub struct TiledLoader;

#[derive(Debug, Error)]
pub enum TiledAssetLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not load Tiled file: {0}")]
    Io(#[from] std::io::Error),
}

impl AssetLoader for TiledLoader {
    type Asset = TiledMap;
    type Settings = ();
    type Error = TiledAssetLoaderError;

    /// Returns a TiledMap loaded from a tmx file
    /// passed into the AssetLoader.
    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _settings: &'a Self::Settings,
        load_context: &'a mut bevy::asset::LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        let mut loader = tiled::Loader::with_cache_and_reader(
            tiled::DefaultResourceCache::new(),
            BytesResourceReader::new(&bytes),
        );
        let map = loader.load_tmx_map(load_context.path()).map_err(|e| {
            std::io::Error::new(ErrorKind::Other, format!("Could not load TMX map: {e}"))
        })?;

        let mut dependencies = Vec::new();
        let mut tilemap_textures = HashMap::default();
        let mut tile_image_offsets = HashMap::default();

        for (tileset_index, tileset) in map.tilesets().iter().enumerate() {
            let tilemap_texture = match &tileset.image {
                None => {
                    {
                        let mut tile_images: Vec<Handle<Image>> = Vec::new();
                        for (tile_id, tile) in tileset.tiles() {
                            if let Some(img) = &tile.image {
                                // The load context path is the TMX file itself. If the file is at the root of the
                                // assets/ directory structure then the tmx_dir will be empty, which is fine.
                                let tmx_dir = load_context
                                    .path()
                                    .parent()
                                    .expect("The asset load context was empty.");
                                let tile_path = tmx_dir.join(&img.source);
                                let asset_path = AssetPath::from(tile_path);
                                log::info!("Loading tile image from {asset_path:?} as image ({tileset_index}, {tile_id})");
                                let texture: Handle<Image> = load_context.load(asset_path.clone());
                                tile_image_offsets
                                    .insert((tileset_index, tile_id), tile_images.len() as u32);
                                tile_images.push(texture.clone());
                                dependencies.push(asset_path);
                            }
                        }

                        TilemapTexture::Vector(tile_images)
                    }
                }
                Some(img) => {
                    // The load context path is the TMX file itself. If the file is at the root of the
                    // assets/ directory structure then the tmx_dir will be empty, which is fine.
                    let tmx_dir = load_context
                        .path()
                        .parent()
                        .expect("The asset load context was empty.");
                    let tile_path = tmx_dir.join(&img.source);
                    let asset_path = AssetPath::from(tile_path);
                    let texture: Handle<Image> = load_context.load(asset_path.clone());
                    dependencies.push(asset_path);

                    TilemapTexture::Single(texture.clone())
                }
            };

            tilemap_textures.insert(tileset_index, tilemap_texture);
        }

        let asset_map = TiledMap {
            map,
            tilemap_textures,
            tile_image_offsets,
        };

        log::info!("Loaded map: {}", load_context.path().display());
        Ok(asset_map)
    }

    fn extensions(&self) -> &[&str] {
        static EXTENSIONS: &[&str] = &["tmx"];
        EXTENSIONS
    }
}

pub fn convert_tiled_to_bevy_pos(tile_pos: TilePos, width: u32) -> TilePos {
    let mapped_y = width - 1 - tile_pos.y;

    TilePos::new(tile_pos.x, mapped_y)
}

pub struct TiledMapInformation<'a> {
    grid_size: &'a TilemapGridSize,
    map_size: &'a TilemapSize,
    map_type: &'a TilemapType,
    map_transform: &'a Transform,
}

impl<'a> TiledMapInformation<'a> {
    pub fn new(
        grid_size: &'a TilemapGridSize,
        map_size: &'a TilemapSize,
        map_type: &'a TilemapType,
        map_transform: &'a Transform,
    ) -> Self {
        TiledMapInformation {
            grid_size,
            map_size,
            map_type,
            map_transform,
        }
    }
}

pub fn tiled_to_bevy_transform(tile_pos: &TilePos, map_info: TiledMapInformation) -> Transform {
    let tiled_to_bevy_pos = convert_tiled_to_bevy_pos(*tile_pos, map_info.map_size.y);
    to_bevy_transform(&tiled_to_bevy_pos, map_info)
}

pub fn to_bevy_transform(tile_pos: &TilePos, map_info: TiledMapInformation) -> Transform {
    let streamer_translation = tile_pos
        .center_in_world(map_info.grid_size, map_info.map_type)
        .extend(map_info.map_transform.translation.z);
    *map_info.map_transform * Transform::from_translation(streamer_translation)
}

pub fn process_loaded_maps(
    mut commands: Commands,
    mut map_events: EventReader<AssetEvent<TiledMap>>,
    maps: Res<Assets<TiledMap>>,
    tile_storage_query: Query<(Entity, &TileStorage)>,
    mut map_query: Query<(
        &Handle<TiledMap>,
        &mut TiledLayersStorage,
        &TilemapRenderSettings,
    )>,
    new_maps: Query<&Handle<TiledMap>, Added<Handle<TiledMap>>>,
) {
    // NOTE: Collect all TiledMap references that have been changed
    let mut changed_maps = Vec::<AssetId<TiledMap>>::default();
    for event in map_events.read() {
        match event {
            AssetEvent::Added { id } => {
                log::info!("Map added!");
                changed_maps.push(*id);
            }
            AssetEvent::Modified { id } => {
                log::info!("Map changed!");
                changed_maps.push(*id);
            }
            AssetEvent::Removed { id } => {
                log::info!("Map removed!");
                // if mesh was modified and removed in the same update, ignore the modification
                // events are ordered so future modification events are ok
                changed_maps.retain(|changed_handle| changed_handle == id);
            }
            _ => continue,
        }
    }

    // NOTE: Include all TiledMap references that have been added as well.
    //
    // If we have new map entities add them to the changed_maps list.
    for new_map_handle in new_maps.iter() {
        changed_maps.push(new_map_handle.id());
    }

    for changed_map in changed_maps.iter() {
        for (map_handle, mut layer_storage, render_settings) in map_query.iter_mut() {
            // only deal with currently changed map
            // NOTE: This effectively filters down all TiledMaps to
            // ones we have recorded into the changed_maps collection.
            if map_handle.id() != *changed_map {
                continue;
            }

            // NOTE: Get the TiledMap instance from all known
            // Assets.
            if let Some(tiled_map) = maps.get(map_handle) {
                // TODO: Create a RemoveMap component..
                // NOTE: Despawn _ALL_ currently rendered Tiles.
                for layer_entity in layer_storage.storage.values() {
                    if let Ok((_, layer_tile_storage)) = tile_storage_query.get(*layer_entity) {
                        for tile in layer_tile_storage.iter().flatten() {
                            commands.entity(*tile).despawn_recursive()
                        }
                    }
                    // commands.entity(*layer_entity).despawn_recursive();
                }

                // The TilemapBundle requires that all tile images come exclusively from a single
                // tiled texture or from a Vec of independent per-tile images. Furthermore, all of
                // the per-tile images must be the same size. Since Tiled allows tiles of mixed
                // tilesets on each layer and allows differently-sized tile images in each tileset,
                // this means we need to load each combination of tileset and layer separately.
                for (tileset_index, tileset) in tiled_map.map.tilesets().iter().enumerate() {
                    // NOTE: Load the Texture for each TileMap based on the used Tile Set (Tile
                    // Palette?)
                    let Some(tilemap_texture) = tiled_map.tilemap_textures.get(&tileset_index)
                    else {
                        log::warn!("Skipped creating layer with missing tilemap textures.");
                        continue;
                    };

                    // NOTE: Define the Tile Size based on the Tile Set used.
                    let tile_size = TilemapTileSize {
                        x: tileset.tile_width as f32,
                        y: tileset.tile_height as f32,
                    };

                    // NOTE: Record the Spacing between Tiles
                    // based on the Tile Set used.
                    let tile_spacing = TilemapSpacing {
                        x: tileset.spacing as f32,
                        y: tileset.spacing as f32,
                    };

                    // Once materials have been created/added we need to then create the layers.
                    //
                    // NOTE: For each Layer,
                    for (layer_index, layer) in tiled_map.map.layers().enumerate() {
                        let offset_x = layer.offset_x;
                        let offset_y = layer.offset_y;

                        // NOTE: Filter for all Layers with type Tiles from the Tiled Map.
                        let tiled::LayerType::Tiles(tile_layer) = layer.layer_type() else {
                            log::info!(
                                "Skipping layer {} because only tile layers are supported.",
                                layer.id()
                            );
                            continue;
                        };

                        // NOTE: Filter for all Finite Tile Layers.
                        let tiled::TileLayer::Finite(layer_data) = tile_layer else {
                            log::info!(
                                "Skipping layer {} because only finite layers are supported.",
                                layer.id()
                            );
                            continue;
                        };

                        // NOTE: Capture the size of the current Tile Map,
                        // where Width and Height are measured in Number
                        // of Tiles.
                        let map_size = TilemapSize {
                            x: tiled_map.map.width,
                            y: tiled_map.map.height,
                        };

                        // NOTE: Capture the maximum size of each Tile from
                        // the current Tile Map in pixels.
                        let grid_size = TilemapGridSize {
                            x: tiled_map.map.tile_width as f32,
                            y: tiled_map.map.tile_height as f32,
                        };

                        // NOTE: Determine the Orientation of the Tile Map.
                        let map_type = match tiled_map.map.orientation {
                            tiled::Orientation::Hexagonal => {
                                TilemapType::Hexagon(HexCoordSystem::Row)
                            }
                            tiled::Orientation::Isometric => {
                                TilemapType::Isometric(IsoCoordSystem::Diamond)
                            }
                            tiled::Orientation::Staggered => {
                                TilemapType::Isometric(IsoCoordSystem::Staggered)
                            }
                            tiled::Orientation::Orthogonal => TilemapType::Square,
                        };

                        // NOTE: Hold references to each newly created Tile
                        // from this current Layer.
                        let mut tile_storage = TileStorage::empty(map_size);
                        let layer_entity = commands.spawn_empty().id();

                        for x in 0..map_size.x {
                            for y in 0..map_size.y {
                                // Transform TMX coords into bevy coords.
                                //
                                // NOTE: Flip the Y-Axis from what Bevy expects
                                // to coordinate with Tiled handles coordinates.
                                let mapped_y = tiled_map.map.height - 1 - y;

                                let mapped_x = x as i32;
                                let mapped_y = mapped_y as i32;

                                // NOTE: Pull the Tile from the Tiled Map.
                                let layer_tile = match layer_data.get_tile(mapped_x, mapped_y) {
                                    Some(t) => t,
                                    None => {
                                        continue;
                                    }
                                };
                                // NOTE: Filter out all Layer Tiles that are not
                                // dealing with the currently loaded Tile Set (Palette).
                                if tileset_index != layer_tile.tileset_index() {
                                    continue;
                                }
                                // NOTE: Pulls Properties about Layer Tile.
                                let layer_tile_data =
                                    match layer_data.get_tile_data(mapped_x, mapped_y) {
                                        Some(d) => d,
                                        None => {
                                            continue;
                                        }
                                    };

                                // NOTE: Get the Texture used for the Layer Tile in
                                // question.
                                let texture_index = match tilemap_texture {
                                    TilemapTexture::Single(_) => layer_tile.id(),
                                    TilemapTexture::Vector(_) =>
                                        *tiled_map.tile_image_offsets.get(&(tileset_index, layer_tile.id()))
                                        .expect("The offset into to image vector should have been saved during the initial load."),
                                    _ => unreachable!()
                                };

                                // NOTE: Spawn the Layer Tile in Bevy Coordinates.
                                let tile_pos = TilePos { x, y };
                                let tile_entity = commands
                                    .spawn((
                                        TileBundle {
                                            position: tile_pos,
                                            tilemap_id: TilemapId(layer_entity),
                                            texture_index: TileTextureIndex(texture_index),
                                            flip: TileFlip {
                                                x: layer_tile_data.flip_h,
                                                y: layer_tile_data.flip_v,
                                                d: layer_tile_data.flip_d,
                                            },
                                            ..Default::default()
                                        },
                                        LayerNumber(layer_index),
                                    ))
                                    .id();
                                // NOTE: Record the recently spawned Layer Tile
                                // into Tile Storage.
                                tile_storage.set(&tile_pos, tile_entity);
                            }
                        }

                        // NOTE: Spawn the Tiled Map Layer as a whole.
                        commands.entity(layer_entity).insert(TilemapBundle {
                            grid_size,
                            size: map_size,
                            storage: tile_storage,
                            texture: tilemap_texture.clone(),
                            tile_size,
                            spacing: tile_spacing,
                            transform: get_tilemap_center_transform(
                                &map_size,
                                &grid_size,
                                &map_type,
                                layer_index as f32,
                            ) * Transform::from_xyz(offset_x, -offset_y, 0.0),
                            map_type,
                            render_settings: *render_settings,
                            ..Default::default()
                        });

                        // NOTE: Record the recently spawned Tiled Map Layer
                        // into Layer Storage.
                        layer_storage
                            .storage
                            .insert(layer_index as u32, layer_entity);
                    }
                }
            }
        }
    }
}

/// Returns a loaded Tiled Map from
/// the provided Path.
fn load_tmx_map(tiled_map_path: &Path) -> TiledMap {
    let mut loader = Loader::new();
    let map = loader
        .load_tmx_map(tiled_map_path)
        .expect("load_tmx_map: Unable to load tmx map.");

    let tilemap_textures = HashMap::default();
    let tile_image_offsets = HashMap::default();

    TiledMap {
        map,
        tilemap_textures,
        tile_image_offsets,
    }
}

/// Loads all necessary information to represent
/// each Tile from a Tiled (.tmx) file.
/// NOTE: For Integration Testing purposes only.
pub fn spawn_tiles_from_tiledmap(mut commands: Commands) {
    // NOTE: Get the TiledMap instance from all known
    // Assets.
    let mut tiled_map_path = PathBuf::new();
    // This has to happen in situations where we're running with the debugger after doing something like
    // `ln -s ~/path/to/task_masker/assets assets`
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        tiled_map_path.push(manifest_dir);
    }
    tiled_map_path.push("assets/TM_map.tmx");

    let tiled_map = load_tmx_map(&tiled_map_path);
    for (tileset_index, tileset) in tiled_map.map.tilesets().iter().enumerate() {
        // NOTE: Define the Tile Size based on the Tile Set used.
        let tile_size = TilemapTileSize {
            x: tileset.tile_width as f32,
            y: tileset.tile_height as f32,
        };

        // NOTE: Record the Spacing between Tiles
        // based on the Tile Set used.
        let tile_spacing = TilemapSpacing {
            x: tileset.spacing as f32,
            y: tileset.spacing as f32,
        };

        // Once materials have been created/added we need to then create the layers.
        //
        // NOTE: For each Layer,
        for (layer_index, layer) in tiled_map.map.layers().enumerate() {
            let offset_x = layer.offset_x;
            let offset_y = layer.offset_y;

            // NOTE: Filter for all Layers with type Tiles from the Tiled Map.
            let tiled::LayerType::Tiles(tile_layer) = layer.layer_type() else {
                log::info!(
                    "Skipping layer {} because only tile layers are supported.",
                    layer.id()
                );
                continue;
            };

            // NOTE: Filter for all Finite Tile Layers.
            let tiled::TileLayer::Finite(layer_data) = tile_layer else {
                log::info!(
                    "Skipping layer {} because only finite layers are supported.",
                    layer.id()
                );
                continue;
            };

            // NOTE: Capture the size of the current Tile Map,
            // where Width and Height are measured in Number
            // of Tiles.
            let map_size = TilemapSize {
                x: tiled_map.map.width,
                y: tiled_map.map.height,
            };

            // NOTE: Capture the maximum size of each Tile from
            // the current Tile Map in pixels.
            let grid_size = TilemapGridSize {
                x: tiled_map.map.tile_width as f32,
                y: tiled_map.map.tile_height as f32,
            };

            // NOTE: Determine the Orientation of the Tile Map.
            let map_type = match tiled_map.map.orientation {
                tiled::Orientation::Hexagonal => TilemapType::Hexagon(HexCoordSystem::Row),
                tiled::Orientation::Isometric => TilemapType::Isometric(IsoCoordSystem::Diamond),
                tiled::Orientation::Staggered => TilemapType::Isometric(IsoCoordSystem::Staggered),
                tiled::Orientation::Orthogonal => TilemapType::Square,
            };

            let layer_entity = commands.spawn_empty().id();

            for x in 0..map_size.x {
                for y in 0..map_size.y {
                    // Transform TMX coords into bevy coords.
                    //
                    // NOTE: Flip the Y-Axis from what Bevy expects
                    // to coordinate with Tiled handles coordinates.
                    let mapped_y = tiled_map.map.height - 1 - y;

                    let mapped_x = x as i32;
                    let mapped_y = mapped_y as i32;

                    // NOTE: Pull the Tile from the Tiled Map.
                    let layer_tile = match layer_data.get_tile(mapped_x, mapped_y) {
                        Some(t) => t,
                        None => {
                            continue;
                        }
                    };
                    // NOTE: Filter out all Layer Tiles that are not
                    // dealing with the currently loaded Tile Set (Palette).
                    if tileset_index != layer_tile.tileset_index() {
                        continue;
                    }
                    // NOTE: Pulls Properties about Layer Tile.
                    let layer_tile_data = match layer_data.get_tile_data(mapped_x, mapped_y) {
                        Some(d) => d,
                        None => {
                            continue;
                        }
                    };

                    // NOTE: Spawn the Layer Tile in Bevy Coordinates.
                    let tile_pos = TilePos { x, y };
                    commands.spawn((
                        TileBundle {
                            position: tile_pos,
                            tilemap_id: TilemapId(layer_entity),
                            flip: TileFlip {
                                x: layer_tile_data.flip_h,
                                y: layer_tile_data.flip_v,
                                d: layer_tile_data.flip_d,
                            },
                            ..Default::default()
                        },
                        LayerNumber(layer_index),
                    ));
                }
            }

            // NOTE: Spawn the Tiled Map Layer as a whole.
            commands.entity(layer_entity).insert((
                grid_size,
                map_size,
                tile_size,
                tile_spacing,
                get_tilemap_center_transform(&map_size, &grid_size, &map_type, layer_index as f32)
                    * Transform::from_xyz(offset_x, -offset_y, 0.0),
                map_type,
            ));
        }
    }
}
