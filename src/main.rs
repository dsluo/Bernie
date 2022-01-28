use dotenv::dotenv;
use serenity::{
    async_trait,
    prelude::*,
};
use crate::soundboard::{
    Backend,
    backend::BackendProvider,
    database::DatabaseBackend,
};

mod soundboard;

struct Bot<T: BackendProvider> {
    backend: T,
}

#[async_trait]
impl<T: BackendProvider> EventHandler for Bot<T> {}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let token = std::env::var("DISCORD_TOKEN").expect("Expected DISCORD_TOKEN in environment.");

    let backend = Backend::setup().await;

    let bot = Bot {
        backend
    };

    let mut client = Client::builder(&token).event_handler(bot).await.expect("Couldn't connect to Discord");

    client.start().await.unwrap();
}