Feature: Fruit Trees.
    Scenario: A piece of fruit falls to the ground.
        Given a Tiled Map,
        And Fruits are spawned on the Tiled Map,
        When a piece of Fruit is detached from its tree,
        Then the Fruit should be dropped on the ground.

    Scenario: A fallen piece of fruit is claimed by the Streamer.
        Given a Tiled Map,
        And Fruits are spawned on the Tiled Map,
        And a Streamer spawned on the Tiled Map,
        When a piece of fruit is done falling,
        And the Streamer travels to the fallen Fruit,
        Then the Streamer should be at the Fruit's location.
        And the Fruit should be back on its tree.
