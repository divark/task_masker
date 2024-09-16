Feature: An isometric tilemap should be loaded from some map file.
    Scenario: One Tile is found from an isometric Tiled map.
        Given a Tiled map called one_tile_isometric.tmx,
        When the tiles are loaded from the Tiled map,
        Then there should be 1 Tile loaded from the Tiled map.
        And Tile 1 should have a width of 64, and a height of 64.
        And Tile 1 should be at grid coordinates 0, 0.
        And Tile 1 should be at pixel coordinates 0, 0.

    Scenario: One Tile with a texture is found from an isometric Tiled map.
        Given a Tiled map called one_tile_isometric.tmx,
        When the tiles are loaded from the Tiled map,
        Then Tile 1 should have a Texture pointing to entry 1 in terrain_0.png.

    Scenario: Inverting the y-axis of a Tile on a one Tile map should change nothing. 
        Given a Tiled map called one_tile_isometric.tmx,
        When the tiles are loaded from the Tiled map,
        And the y-axis has been inverted for all tiles,
        Then Tile 1 should be at pixel coordinates 0, 0.
