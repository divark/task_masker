Feature: A Chatter, represented as a Bird, can approach the Streamer to speak, and leave when finished.

    Scenario: The Chatter approaches the Streamer.
        Given a Tiled Map
        And a Streamer spawned on the Tiled Map
        And a Chatter spawned on the Tiled Map
        When the Chatter wants to speak
        Then the Chatter will approach the Streamer

    Scenario: The Chatter arrives to the Streamer two tiles away.
        Given a Tiled Map
        And a Streamer spawned on the Tiled Map
        And a Chatter spawned on the Tiled Map
        When the Chatter has approached the Streamer
        Then the Chatter should be two tiles away from the Streamer
        And the Chatter will begin to speak.

    Scenario: The Chatter leaves when finished speaking to the Streamer
        Given a Tiled Map
        And a Streamer spawned on the Tiled Map
        And a Chatter spawned two tiles away from the Streamer 
        When the Chatter is done speaking
        Then the Chatter will leave back to its resting point.
