Feature: A Subscriber, represented as a Fish, can approach the Streamer to speak, and leave when finished.

    Scenario: The Subscriber approaches the Streamer.
        Given a Tiled Map
        And a Streamer spawned on the Tiled Map
        And a Subscriber spawned on the Tiled Map
        When the Subscriber wants to speak
        Then the Subscriber will approach the Streamer

    Scenario: The Subscriber arrives to the coast closest to the Streamer.
        Given a Tiled Map
        And a Streamer spawned on the Tiled Map
        And a Subscriber spawned on the Tiled Map
        When the Subscriber has approached the Streamer
        Then the Subscriber will be on the coast closest to the Streamer
        And the Subscriber will begin to speak

    Scenario: The Subscriber leaves when finished speaking to the Streamer
        Given a Tiled Map
        And a Streamer spawned on the Tiled Map
        And a Subscriber spawned on the Tiled Map
        When the Subscriber has approached the Streamer
        And the Subscriber is done speaking
        Then the Subscriber leaves back to its resting point
