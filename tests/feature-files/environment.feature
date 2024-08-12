Feature: Props from the Environment should animate.
    Scenario: The Campfire is set up to flicker in the wind.
        Given a Tiled Map,
        When the Campfire is spawned on the Tiled Map,
        Then the Campfire should have an animation speed set,
        And the Campfire should have 23 frames to flicker through while animating.
