Feature: Chat Messages mapped for Streamer, Subscriber, and Chatter.
    Scenario: The Streamer posts a chat message.
        Given a Streamer is spawned on the map,
        And the Chatting interface exists,
        When the Streamer sends a chat message,
        Then the Chatting Queue should contain the Streamer's chat message.
