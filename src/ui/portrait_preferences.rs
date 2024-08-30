use bevy::prelude::*;
use sqlite::{Connection, ConnectionThreadSafe};

pub const DEFAULT_SUBSCRIBER_SPRITE_IDX: usize = 210;

#[derive(Resource)]
pub struct PortraitPreferences {
    db_connection: ConnectionThreadSafe,
    default_value: usize,
}

impl PortraitPreferences {
    pub fn new(db_name: String, default_value: usize) -> Self {
        let sqlite_connection = Connection::open_thread_safe(db_name).unwrap();

        let create_table_query = "
            CREATE TABLE IF NOT EXISTS twitch_portrait_preferences (name VARCHAR(25) NOT NULL PRIMARY KEY, preference INTEGER);
        ";

        sqlite_connection.execute(create_table_query).expect(
            "PortraitPreferences::new: Could not create Twitch Portrait Preferences table.",
        );

        Self {
            db_connection: sqlite_connection,
            default_value,
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

        let found_entry = find_preference_statement
            .into_iter()
            .map(|row| row.unwrap())
            .next();

        if let Some(portrait_preference) = found_entry {
            portrait_preference.read::<i64, _>("preference") as usize
        } else {
            self.default_value
        }
    }

    /// Inserts or Updates the Preference for some specified user.
    pub fn set(&mut self, user: String, preference_idx: usize) {
        let insert_query = "
            INSERT INTO twitch_portrait_preferences(name, preference)
            VALUES (?, ?)
            ON CONFLICT(name)
            DO UPDATE SET preference=excluded.preference;
        ";
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
