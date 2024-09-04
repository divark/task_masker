Feature: Any Game Entity should obey the rules of path finding when traveling anywhere.
    Scenario: An UndirectedGraph should exist for the island.
        Given the Tiled Loading module is loaded,
        And the Path Finding module is loaded,
        When the Tiled map is loaded,
        Then there should be an Undirected Graph representing all ground tiles.

    Scenario: A Path should exist between two touching tiles on the island.
        Given the Tiled Loading module is loaded,
        And the Path Finding module is loaded,
        When the Tiled map is loaded,
        Then there should be a Path from the Undirected Graph starting from one tile, going to a neighboring tile.
