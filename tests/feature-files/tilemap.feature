Feature: An isometric tilemap should be loaded from some map file.
    Scenario: One Tile is found from an isometric Tiled map.
        Given a Tiled map called one_tile_isometric.tmx,
        When the tiles are loaded from the Tiled map,
        Then there should be 1 Tile loaded from the Tiled map.
        And Tile 1 should have a width of 32, and a height of 32.
        And Tile 1 should be in logical position 0, 0.
        And Tile 1 should be at pixel coordinates 0, 0.

    Scenario: One Tile with a texture is found from an isometric Tiled map.
        Given a Tiled map called one_tile_isometric.tmx,
        When the tiles are loaded from the Tiled map,
        Then Tile 1 should have a Texture pointing to Iso_Tiles32x32_Fox.png.
        And Tile 1 should have a Texture pointing to spritesheet entry 4.
