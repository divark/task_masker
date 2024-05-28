Feature: A Streamer can walk to any point on the map.

    Scenario: The Streamer moves to a lower point on the island.
        Given a 5x5 Island Map
        And the Streamer spawned at a higher point on the island
        And a desired point on the island that is lower in height from the Streamer 
        When the Streamer travels to the desired point
        Then the Streamer should be at that desired point

    Scenario: The Streamer moves to a higher point on the island.
        Given a 5x5 Island Map
        And the Streamer spawned at a lower point on the island
        And a desired point on the island that is higher in height from the Streamer
        When the Streamer travels to the desired point
        Then the Streamer should be at that desired point

    Scenario: The Streamer moves to a point equal in height on the island.
        Given a 5x5 Island Map
        And the Streamer spawned at a level point on the island
        And a desired point on the island that is equal in height from the Streamer
        When the Streamer travels to the desired point
        Then the Streamer should be at that desired point

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

