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
        Then Tile 0, 0, 0 should have a Texture pointing to entry 1 in terrain_0.png.

    Scenario: The highest Tile with a texture is found on a two-layer, isometric Tiled map.
        Given a Tiled map called two_tiles_layered.tmx,
        When the tiles are loaded from the Tiled map,
        Then Tile 0, 0, 0 should have a Texture pointing to entry 1 in terrain_0.png.
        And Tile 0, 0, 1 should have a Texture pointing to entry 60 in terrain_0.png.

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
