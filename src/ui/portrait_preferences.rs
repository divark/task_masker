use bevy::prelude::*;
use sqlite::{Connection, ConnectionThreadSafe};

#[derive(Resource)]
pub struct PortraitPreferences {
    db_connection: ConnectionThreadSafe,
}

impl PortraitPreferences {
    pub fn new(db_name: String) -> Self {
        let sqlite_connection = Connection::open_thread_safe(db_name).unwrap();

        let create_table_query = "
            CREATE TABLE IF NOT EXISTS twitch_portrait_preferences (name VARCHAR(25) NOT NULL PRIMARY KEY, preference INTEGER);
        ";

        sqlite_connection.execute(create_table_query).expect(
            "PortraitPreferences::new: Could not create Twitch Portrait Preferences table.",
        );

        Self {
            db_connection: sqlite_connection,
        }
    }

    /// Returns the index found for some specified user, or
    /// the default if none was not.
    pub fn get(&self, user: String) -> usize {
        let find_preference_query =
            "SELECT preference FROM twitch_portrait_preferences WHERE name = ?";
        let mut find_preference_statement = self
            .db_connection
            .prepare(find_preference_query)
            .expect("PortraitPreferences get: Could not make Select statement from query.");

        find_preference_statement
            .bind((1, user.as_str()))
            .expect("PortraitPreferences get: Could not bind user string.");
        find_preference_statement
            .next()
            .expect("PortraitPreferences get: Could not process find_preference statement.");

        if let Ok(portrait_preference) = find_preference_statement.read::<i64, _>("preference") {
            return portrait_preference as usize;
        } else {
            return 0;
        }
    }

    /// Updates the Preference for some specified user.
    pub fn set(&mut self, user: String, preference_idx: usize) {
        let insert_query = "INSERT INTO twitch_portrait_preferences VALUES (?, ?);";
        let mut insert_statement = self
            .db_connection
            .prepare(insert_query)
            .expect("PortraitPreferences set: Could not make Insert Statement from query.");

        insert_statement
            .bind((1, user.as_str()))
            .expect("PortraitPreferences set: Could not bind user string.");
        insert_statement
            .bind((2, preference_idx as i64))
            .expect("PortraitPreferences set: Could not bind preference idx.");

        insert_statement
            .next()
            .expect("PortraitPreferences set: Could not process insert_statement.");
    }
}
