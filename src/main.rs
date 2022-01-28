use dotenv::dotenv;
use serenity::{
    async_trait,
    prelude::*,
};

use crate::soundboard::database::DatabaseBackend;

use crate::soundboard::SoundboardBackend;

mod soundboard;

struct Bot<T: SoundboardBackend> {
    backend: T,
}

#[async_trait]
impl<T: SoundboardBackend> EventHandler for Bot<T> {}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let token = std::env::var("DISCORD_TOKEN").expect("Expected DISCORD_TOKEN in environment.");

    let uri = std::env::var("DATABASE_URL").expect("Expected DATABASE_URL in environment.");

    let backend = DatabaseBackend::new(&uri).await.expect("Couldn't connect to backend.");

    let bot = Bot {
        backend
    };

    let mut client = Client::builder(&token).event_handler(bot).await.expect("Couldn't connect to Discord");

    client.start().await.unwrap();
}
