Feature: Fruit Trees.
    Scenario: A piece of Fruit can fall to the ground.
        Given a Tiled Map,
        And Fruits are spawned on the Tiled Map,
        When some Fruit is requested to drop,
        Then the Fruit should be heading towards the ground.

    Scenario: A Streamer will move to the dropped Fruit.
        Given a Tiled Map,
        And Fruits are spawned on the Tiled Map,
        And a Streamer spawned on the Tiled Map,
        When the Fruit has been dropped, 
        Then the Streamer should be heading towards the fallen Fruit's position.

    Scenario: A piece of Fruit is claimed by the Streamer.
        Given a Tiled Map,
        And Fruits are spawned on the Tiled Map,
        And a Streamer spawned on the Tiled Map,
        When the Fruit has been dropped,
        And the Streamer is over the dropped Fruit,
        Then the dropped Fruit will disappear.

    Scenario: A piece of Fruit that is claimed respawns on its tree.
        Given a Tiled Map,
        And Fruits are spawned on the Tiled Map,
        And a Streamer spawned on the Tiled Map,
        When the Fruit has been dropped,
        And the Fruit has been picked up by the Streamer,
        Then the Fruit will re-appear back on its tree.
