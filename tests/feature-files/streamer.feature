Feature: A Streamer can walk to any point on the map.
    Scenario: A Streamer travels to a lower location.
        Given a Tiled Map,
        And a Streamer spawned on the Tiled Map,
        When the Streamer is requested to travel to a lower location,
        Then the Streamer will arrive at the lower location after traveling there.

    Scenario: A Streamer travels to an equal location.
        Given a Tiled Map,
        And a Streamer spawned on the Tiled Map,
        When the Streamer is requested to travel to an equal in height location,
        Then the Streamer will arrive at the equal in height location after traveling there.

    Scenario: A Streamer travels to a higher location.
        Given a Tiled Map,
        And a Streamer spawned on the Tiled Map,
        When the Streamer is requested to travel to a higher location,
        Then the Streamer will arrive at the higher location after traveling there.

    Scenario: Changing the Status to Online moves the Streamer to the campfire.
        Given a Tiled Map,
        And a Streamer spawned on the Tiled Map,
        And the Campfire spawned on the Tiled Map,
        When the Status changes to Online,
        And the Streamer is done traveling,
        Then the Streamer should be to the left of the campfire.
        And the Streamer should be facing right towards the campfire.

    Scenario: Changing the Status to Away moves the Streamer inside the cave.
        Given a Tiled Map,
        And a Streamer spawned on the Tiled Map,
        When the Status changes to Away,
        And the Streamer is done traveling,
        Then the Streamer should be in the cave.
