Feature: A Streamer can walk to any point on the map.

    Scenario: A Tiled Tile Position maps to a Bevy Tile Position
        Given a 5x5 Island Map
        """
            Legend:
                ~ = Water
                o = Land

            ~~~~~
            ~ooo~
            ~ooo~
            ~ooo~
            ~~~~~
        """
        And a Tiled Tile Position
        """
            Legend:
                ~ = Water
                o = Land
                x = Tiled Tile Position (1, 1)

            ~~~~~
            ~xoo~
            ~ooo~
            ~ooo~
            ~~~~~
        """
        And an expected Bevy Tile Position
        """
            Legend:
                ~ = Water
                o = Land
                b = Bevy Tile Position (1, 3)

            ~~~~~
            ~boo~
            ~ooo~
            ~ooo~
            ~~~~~
        """
        When I convert the Tiled Tile Position into a Bevy Tile Position
        Then the converted Tile Position should match the expected Bevy Tile Position

    Scenario: A Bevy Tile Position maps to a Tiled Tile Position
        Given a 5x5 Island Map
        And a Bevy Tile Position
        And an expected Tiled Tile Position
        When I convert the Bevy Tile Position into a Tiled Tile Position
        Then the converted Tile Position should match the expected Tiled Tile Position
