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
        Then the Chatter will be two tiles away from the Streamer
        And the Chatter will begin to speak

    Scenario: The Chatter leaves when finished speaking to the Streamer
        Given a Tiled Map
        And a Streamer spawned on the Tiled Map
        And a Chatter spawned on the Tiled Map
        And the Chatting interface exists
        When the Chatter has approached the Streamer
        And the Chatter is done speaking
        Then the Chatter leaves back to its resting point

    Scenario: A Chatter should not leave until done speaking.
        Given a Tiled Map
        And a Streamer spawned on the Tiled Map
        And a Chatter spawned on the Tiled Map
        And the Chatting interface exists
        When the Chatter sends a long chat message
        And the Chatter is almost done speaking to the Streamer
        Then the Chatter should still be speaking

    Scenario: A Chatter with two back-to-back messages will stick around.
	Given a Tiled Map
        And a Streamer spawned on the Tiled Map
        And a Chatter spawned on the Tiled Map
        And the Chatting interface exists
        When the Chatter sends a chat message
        And the Chatter sends a different chat message
        And the Chatter has approached the Streamer
        And the Chatter has finished speaking the first message to the Streamer
        Then the Chatter should not be waiting to leave
        And the Chatter should start speaking from the next chat message

