Feature: Preferences for Subscriber Portraits
    Scenario: The default portrait is returned for someone with no preference.
        Given a portrait preference recorder,
        When I ask for the portrait preference for a user with no preference,
        Then the index of the default portrait should be returned.

    Scenario: A portrait preference is saved for someone with no preference.
        Given a portrait preference recorder,
        When I change a portrait preference for a user,
        Then the portrait preference should be saved for the user.
