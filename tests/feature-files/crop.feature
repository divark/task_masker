Feature: Crops.
    Scenario: A Streamer will move to the grown Crop.
        Given a Tiled Map,
        And Crops are spawned on the Tiled Map,
        And a Streamer spawned on the Tiled Map,
        When the Crop has been fully grown, 
        Then the Streamer should be heading towards the grown Crop's position.

    Scenario: A piece of Crop is claimed by the Streamer.
        Given a Tiled Map,
        And Crops are spawned on the Tiled Map,
        And a Streamer spawned on the Tiled Map,
        When the Crop has been fully grown,
        And the Streamer is over the grown Crop,
        Then the Crop will be replanted.
