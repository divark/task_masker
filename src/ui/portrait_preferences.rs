use bevy::prelude::*;
use sqlite::{Connection, ConnectionThreadSafe};

#[derive(Resource)]
pub struct PortraitPreferences {
    db_connection: ConnectionThreadSafe,
}

impl PortraitPreferences {
    pub fn new(db_name: String) -> Self {
        let sqlite_connection = Connection::open_thread_safe(db_name).unwrap();

        Self {
            db_connection: sqlite_connection,
        }
    }

    /// Returns the index found for some specified user, or
    /// the default if none was not.
    pub fn get(&self, user: String) -> usize {
        0
    }
}
