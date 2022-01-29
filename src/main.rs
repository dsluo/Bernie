use dotenv::dotenv;
use crate::soundboard::{
    Backend,
    COMMANDS,
    BackendProvider,
};

mod soundboard;

pub type Error = Box<dyn std::error::Error + Sync + Send>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

pub struct Data {
    backend: Backend
}

/// Register slash commands.
/// No argument to register with current guild; "global" as argument to register globally.
#[poise::command(prefix_command, hide_in_help)]
async fn register(ctx: Context<'_>, #[flag] global: bool) -> Result<(), Error> {
    poise::builtins::register_application_commands(ctx, global).await?;

    Ok(())
}

/// Send help.
#[poise::command(slash_command, prefix_command)]
async fn help(
    ctx: Context<'_>,
    #[description = "Help"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    command: Option<String>,
) -> Result<(), Error> {
    poise::builtins::help(
        ctx,
        command.as_deref(),
        poise::builtins::HelpConfiguration {
            ..Default::default()
        })
        .await?;
    Ok(())
}

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    todo!()
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let token = std::env::var("DISCORD_TOKEN").expect("Expected DISCORD_TOKEN in environment.");

    let mut commands = vec![register(), help()];
    commands.extend(Vec::from(COMMANDS.map(|f| f())));
    let commands = commands;


    let options = poise::FrameworkOptions {
        commands,
        prefix_options: poise::PrefixFrameworkOptions {
            mention_as_prefix: true,
            ..Default::default()
        },
        on_error: |error| Box::pin(async move {
            if let Err(e) = poise::builtins::on_error(error).await {
                log::error!("Error while handling error: {}", e);
            }
        }),
        ..Default::default()
    };

    let backend = Backend::setup().await;

    poise::Framework::build()
        .token(token)
        .user_data_setup(move |_ctx, _ready, _framework| Box::pin(async move {
            Ok(Data {
                backend
            })
        }))
        .options(options)
        .run()
        .await
        .unwrap();
}
