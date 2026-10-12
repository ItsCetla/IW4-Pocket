use bevy::prelude::*;

/// Lines queued at process-launch / map-launch time and drained into the console
/// FIFO once the next world is being installed. Not drained during menu navigation,
/// so anything whose first token is `map` is already excluded by the caller.
#[derive(Resource, Default)]
pub struct StartupCommands {
    pub lines: Vec<String>,
}

#[derive(Resource, Default)]
pub struct PendingConsoleLines(pub Vec<String>);

/// Bot count the frontend lobby asked for when it started the next swap.
/// Consumed once by `apply_prepared_match` and cleared when the match installs.
#[derive(Resource, Default)]
pub struct PendingBotAdd {
    pub count: u32,
}
