use crate::Game;

use super::map::MapUpdater;

impl<'g> Game<'g> {
    pub(crate) fn setup_basic_events(&mut self) {
        // move the map Downward
        self.add_event(
            MapUpdater, // Exclusive type (implements IntoWorldEvent) to update map
        );
    }
}
