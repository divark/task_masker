Feature: Chat Messages mapped for Streamer, Subscriber, and Chatter.
    Scenario: The Streamer posts a chat message.
        Given a Streamer is spawned on the map,
        And the Chatting interface exists,
        When the Streamer sends a chat message,
        Then the Chatting Queue should contain the Streamer's chat message.

    Scenario: The Chatter posts a chat message.
	Given a Chatter is spawned on the map,
	And the Chatting interface exists,
	When the Chatter sends a chat message,
	Then the Chatting Queue should contain the Chatter's chat message.

    Scenario: The Subscriber posts a chat message.
	Given a Subscriber is spawned on the map,
	And the Chatting interface exists,
	When the Subscriber sends a chat message,
	Then the Chatting Queue should contain the Subscriber's chat message.

    Scenario: The Chatter posts a chat message, and the Streamer does at the same time.
	Given a Streamer is spawned on the map,
	And a Chatter is spawned on the map,
	When the Chatter sends a chat message,
	And the Streamer sends a chat message,
	Then the Chatting Queue should have the Streamer's chat message as the top priority.
