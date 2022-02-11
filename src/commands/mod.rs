use poise::Command;

use crate::{Data, Error};

mod meta;
mod playbacks;
mod sounds;

use playbacks::{history, play, random, stop};
use sounds::{add, list, remove, rename};

pub use playbacks::TrackManager;

pub const COMMANDS: [fn() -> Command<Data, Error>; 8] =
    [play, random, stop, add, list, rename, remove, history];
