use dotenv::dotenv;
use sqlx::PgPool;

mod commands;

use commands::COMMANDS;

pub type Error = Box<dyn std::error::Error + Sync + Send>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

pub struct Data {
    db: PgPool
}

const OAUTH_SCOPES: [OAuth2Scope; 2] = [OAuth2Scope::Bot, OAuth2Scope::ApplicationsCommands];

const PERMISSIONS: [Permissions; 2] = [Permissions::SPEAK, Permissions::CONNECT];

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
        },
    )
    .await?;
    Ok(())
}

/// Get the invite for this bot.
#[poise::command(slash_command, prefix_command)]
async fn invite(ctx: Context<'_>) -> Result<(), Error> {
    let bot = ctx.discord().cache.current_user();
    let http = &ctx.discord().http;

    let permissions: Permissions = PERMISSIONS
        .into_iter()
        .reduce(|total, current| total | current)
        .unwrap();

    let invite = bot
        .invite_url_with_oauth2_scopes(http, permissions, &OAUTH_SCOPES)
        .await?;

    ctx.say(invite).await?;

    Ok(())
}

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::Setup { error } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx } => {
            log::error!("Error in command `{}`: {:?}", ctx.command().name, error);
        },
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                log::error!("Error while handling error: {:?}", e);
            }
        }
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();
    dotenv().ok();

    let token = std::env::var("DISCORD_TOKEN").expect("Expected DISCORD_TOKEN in environment.");

    let uri = std::env::var("DATABASE_URL").expect("Expected DATABASE_URL in environment.");
    let db = PgPool::connect(&uri).await.expect("Couldn't connect to database.");

    let mut commands = vec![register(), help(), invite()];
    commands.extend(Vec::from(COMMANDS.map(|f| f())));


    let options = poise::FrameworkOptions {
        commands,
        prefix_options: poise::PrefixFrameworkOptions {
            mention_as_prefix: true,
            ..Default::default()
        },
        on_error: |error| Box::pin(on_error(error)),
        ..Default::default()
    };

    poise::Framework::build()
        .token(token)
        .user_data_setup(move |_ctx, _ready, _framework| Box::pin(async move {
            Ok(Data {
                db
            })
        }))
        .options(options)
        .run()
        .await
        .unwrap();
}
