Feature: Chat Messages mapped for Streamer, Subscriber, and Chatter.
    Scenario: The Streamer posts a chat message.
        Given a Streamer is spawned on the map,
        And the Chatting interface exists,
        When the Streamer sends a chat message,
        Then the Streamer should be speaking currently.

    Scenario: The Chatter posts a chat message.
        Given a Chatter is spawned on the map,
        And the Chatting interface exists,
        When the Chatter sends a chat message,
        Then the Chatter should be speaking currently.

    Scenario: The Subscriber posts a chat message.
        Given a Subscriber is spawned on the map,
        And the Chatting interface exists,
        When the Subscriber sends a chat message,
        Then the Subscriber should be speaking currently.

    Scenario: The Chatter posts a chat message, and the Streamer does at the same time.
        Given a Streamer is spawned on the map,
        And a Chatter is spawned on the map,
        And the Chatting interface exists,
        When the Subscriber sends a chat message,
        And the Chatter sends a chat message,
        And the Streamer sends a chat message,
        Then the Streamer should be speaking next.

    Scenario: A Chat Message is read one character at a time.
        Given a Streamer is spawned on the map,
        And the Chatting interface exists,
        When the Streamer sends a chat message,
        And the first five characters of the chat message has been read,
        Then the Chat UI should contain the first five characters typed from the Chat Message.

    Scenario: A fully read Chat Message is no longer displayed after 5 seconds.
        Given a Streamer is spawned on the map,
        And the Chatting interface exists,
        When the Streamer sends a chat message,
        And the chat message has been fully read,
        And the wait time is up,
        Then the Chat Message should no longer be present,
        And the Chat UI should be hidden.

