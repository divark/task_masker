Feature: An isometric tilemap should be loaded from some map file.
    Scenario: One Tile is found from an isometric Tiled map.
        Given a Tiled map called one_tile_isometric.tmx,
        When the tiles are loaded from the Tiled map,
        Then there should be 1 Tile loaded from the Tiled map.
        And Tile 0, 0, 0 should have a width of 64, and a height of 64.
        And Tile 0, 0, 0 should be at grid coordinates 0, 0, 0.
        And Tile 0, 0, 0 should be at pixel coordinates 0, 0, 0.

    Scenario: One Tile with a texture is found from an isometric Tiled map.
        Given a Tiled map called one_tile_isometric.tmx,
        When the tiles are loaded from the Tiled map,
        Then Tile 0, 0, 0 should have a Texture using the spritesheet file terrain_0.png.
        And Tile 0, 0, 0 should have a Texture pointing to sprite entry 1.

    Scenario: A Tile with a texture has the number of rows and columns in its spritesheet.
        Given a Tiled map called one_tile_isometric.tmx,
        When the tiles are loaded from the Tiled map,
        Then Tile 0, 0, 0 should have 16 rows in its' Texture.
        And Tile 0, 0, 0 should have 10 columns in its' Texture.

    Scenario: The highest Tile with a texture is found on a two-layer, isometric Tiled map.
        Given a Tiled map called two_tiles_layered.tmx,
        When the tiles are loaded from the Tiled map,
        Then Tile 0, 0, 0 should have a Texture using the spritesheet file terrain_0.png.
        And Tile 0, 0, 0 should have a Texture pointing to sprite entry 1.
        And Tile 0, 0, 1 should have a Texture using the spritesheet file terrain_0.png.
        And Tile 0, 0, 1 should have a Texture pointing to sprite entry 60.

    Scenario: Inverting the y-axis of a Tile on a one Tile map should change nothing. 
        Given a Tiled map called one_tile_isometric.tmx,
        When the tiles are loaded from the Tiled map,
        And the y-axis has been inverted for all tiles,
        Then Tile 0, 0, 0 should be at pixel coordinates 0, 0, 0.

    Scenario: Inverting the y-axis with two vertical Tiles should change their y coordinates.
        Given a Tiled map called two_vertical_tiles_isometric.tmx,
        When the tiles are loaded from the Tiled map,
        And the y-axis has been inverted for all tiles,
        Then Tile 0, 0, 0 should be at pixel coordinates 0, 64, 0.
        And Tile 0, 1, 0 should be at pixel coordinates 0, 0, 0.

    Scenario: Converting two Tile coordinates to isometric should change their x and y values.
        Given a Tiled map called two_vertical_tiles_isometric.tmx,
        When the tiles are loaded from the Tiled map,
        And the tile coordinates have been converted to isometric,
        Then Tile 0, 0, 0 should be at pixel coordinates 0, 0, 0.
        And Tile 0, 1, 0 should be at pixel coordinates -64, 32, 0.

    Scenario: The highest Tile is found on a two-layer, isometric Tiled map.
        Given a Tiled map called two_tiles_layered.tmx,
        When the tiles are loaded from the Tiled map,
        Then there should be 2 Tiles loaded from the Tiled map.
        And Tile 0, 0, 0 should have a width of 64, and a height of 64.
        And Tile 0, 0, 0 should be at grid coordinates 0, 0, 0.
        And Tile 0, 0, 0 should be at pixel coordinates 0, 0, 0.
        And Tile 0, 0, 1 should have a width of 64, and a height of 64.
        And Tile 0, 0, 1 should be at grid coordinates 0, 0, 1.
        And Tile 0, 0, 1 should be at pixel coordinates 0, 0, 1.

    Scenario: Vertical layer offsets in isometric coordinates are honored for some isometric Tiled map.
        Given a Tiled map called two_vertical_tiles_with_offset.tmx,
        When the tiles are loaded from the Tiled map,
        And the tile coordinates have been converted to isometric,
        Then Tile 0, 0, 0 should be at pixel coordinates 0, 0, 0.
        And Tile 0, 0, 1 should be at pixel coordinates 0, -32, 1.

    Scenario: Drawing offsets in isometric coordinates are honored for some isometric Tiled map.
        Given a Tiled map called two_vertical_tiles_with_drawing_offset.tmx,
        When the tiles are loaded from the Tiled map,
        And the tile coordinates have been converted to isometric,
        Then Tile 0, 0, 0 should be at pixel coordinates 0, 0, 0.
        And Tile 0, 0, 1 should be at pixel coordinates 0, -32, 1.

    Scenario: Blank Tiles have no texture for a tile.
        Given a Tiled map called blank_tile_in_corner.tmx,
        When the tiles are loaded from the Tiled map,
        Then Tile 0, 0, 0 should have a Texture using the spritesheet file terrain_0.png.
        And Tile 0, 1, 0 should have a Texture using the spritesheet file terrain_0.png.
        And Tile 1, 0, 0 should have a Texture using the spritesheet file terrain_0.png.
        And Tile 1, 1, 0 should not have a Texture.

    Scenario: Render Tiles are created from Tiles with a texture.
        Given a Tiled map called blank_tile_in_corner.tmx,
        When the tiles are loaded from the Tiled map,
        Then the number of render tiles created should match the number of Tiles with a Texture.
    
    Scenario: The width and height of a Tile is different than a Tile texture's width and height.
        Given a Tiled map called two_vertical_tiles_with_offset.tmx,
        When the tiles are loaded from the Tiled map,
        Then Tile 0, 0, 0 should have a width of 64, and a height of 32.
        And Tile 0, 0, 0's Texture should have a width of 64, and a height of 64.

    Scenario: A Tiled Isometric map is honored with a 2:1 ratio.
        Given a Tiled map called blank_tile_in_corner.tmx,
        When the tiles are loaded from the Tiled map,
        And the tile coordinates have been converted to isometric, 
        Then Tile 0, 0, 0 should be at pixel coordinates 0, 0, 0.
        And Tile 1, 0, 0 should be at grid coordinates 1, 0, 0.
        And Tile 1, 0, 0 should be at pixel coordinates 32, 16, 0.
        And Tile 0, 1, 0 should be at pixel coordinates -32, 16, 0.

    Scenario: A single-layer Tiled isometric map has the correct tile depths.
        Given a Tiled map called single_layer_isometric.tmx,
        When the tiles are loaded from the Tiled map,
        And the tile coordinates have been converted to isometric,
        And the tile coordinates have been y-sorted,
        Then Tile 0, 1, 0 should be higher than Tile 0, 0, 0.
        And Tile 1, 0, 0 should be higher than Tile 0, 0, 0.
        And Tile 1, 1, 0 should be higher than Tile 0, 1, 0.
        And Tile 1, 1, 0 should be higher than Tile 1, 0, 0.

    Scenario: A smaller Tile should be rendered in the right place.
        Given a Tiled map called smaller_tile_on_map.tmx,
        When the tiles are loaded from the Tiled map,
        And the tile coordinates have been converted to isometric,
        Then Tile 0, 0, 1 should be at pixel coordinates 0, -32, 1.

    Scenario: A height map with normal terrain can be calculated.
        Given a Tiled map called heighted_map.tmx,
        When the tiles are loaded from the Tiled map,
        And the tiles have been converted into a height map,
        Then Tile 0, 0 should have a height of 2.
        And Tile 0, 1 should have a height of 1.
        And Tile 1, 0 should have a height of 1.
        And Tile 1, 1 should have a height of 0.
