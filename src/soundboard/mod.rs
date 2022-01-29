pub use backend::BackendProvider;
pub use commands::COMMANDS;

#[cfg(feature = "database")]
use self::database::DatabaseBackend;

mod types;
mod backend;
mod commands;

#[cfg(feature = "database")]
mod database;

#[cfg(feature = "database")]
pub type Backend = DatabaseBackend;
