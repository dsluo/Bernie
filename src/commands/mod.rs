use poise::Command;

use crate::{Data, Error};

mod meta;
mod sounds;
mod playbacks;

use playbacks::{play, random, stop, history};
use sounds::{add, list, rename, remove};

pub const COMMANDS: [fn() -> Command<Data, Error>; 8] =
    [play, random, stop, add, list, rename, remove, history];
