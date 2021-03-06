use std::path::{Path, PathBuf};

use dotenv::dotenv;
use serenity::model::prelude::*;
use sqlx::PgPool;

use commands::{TrackManager, COMMANDS};
use tokio::sync::OnceCell;

mod commands;

pub type Error = anyhow::Error;
pub type Context<'a> = poise::Context<'a, Data, Error>;

#[derive(Debug)]
pub struct Data {
    db: PgPool,
    storage_dir: PathBuf,
    track_manager: TrackManager,
}

impl Data {
    pub fn new<P: AsRef<Path>>(db: PgPool, storage_dir: P) -> Self {
        Self {
            db,
            storage_dir: storage_dir.as_ref().to_path_buf(),
            track_manager: TrackManager::new(),
        }
    }
}

const OAUTH_SCOPES: [OAuth2Scope; 2] = [OAuth2Scope::Bot, OAuth2Scope::ApplicationsCommands];

const PERMISSIONS: [Permissions; 2] = [Permissions::SPEAK, Permissions::CONNECT];

/// Register slash commands.
/// No argument to register with current guild; `global` as argument to register globally.
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

static INVITE: OnceCell<String> = OnceCell::const_new();

async fn get_invite(ctx: Context<'_>) -> Result<&'static String, Error> {
    INVITE
        .get_or_try_init(|| async {
            let bot = ctx.discord().cache.current_user();
            let http = &ctx.discord().http;

            let permissions: Permissions = PERMISSIONS
                .into_iter()
                .reduce(|total, current| total | current)
                .unwrap();

            let invite = bot
                .invite_url_with_oauth2_scopes(http, permissions, &OAUTH_SCOPES)
                .await?;

            Ok(invite)
        })
        .await
}

/// Get the invite for this bot.
#[poise::command(slash_command, prefix_command)]
async fn invite(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say(get_invite(ctx).await?).await?;

    Ok(())
}
/// Get info about the bot.
#[poise::command(slash_command, prefix_command)]
async fn about(ctx: Context<'_>) -> Result<(), Error> {
    // todo: add library information.
    let author = {
        // we're (i'm) going to assume only one author.
        let original = env!("CARGO_PKG_AUTHORS");
        if let Some((name, _)) = original.split_once(" ") {
            name
        } else {
            original
        }
        .trim()
    };

    let name = env!("CARGO_PKG_NAME");
    let version = env!("CARGO_PKG_VERSION");
    let repo = env!("CARGO_PKG_REPOSITORY");

    let about = format!(
        "{name} v{version}\n\
        by `{author}`\n\
        source: {repo}",
    );

    let message = if let Ok(invite) = get_invite(ctx).await {
        format!("{about}\ninvite: {invite}")
    } else {
        about.to_owned()
    };

    ctx.say(message).await?;
    Ok(())
}

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    log::warn!("Encountered error: {error:#?}");
    match error {
        poise::FrameworkError::Setup { error } => panic!("Failed to start bot: {error:?}"),
        // poise::FrameworkError::Command { error, ctx } => {
        //     log::error!("Error in command `{}`: {error:#?}", ctx.command().name);
        // }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                log::error!("Error while handling error: {e:#?}");
            }
        }
    }
}

async fn do_main() {
    dotenv().ok();
    env_logger::init();

    log::info!(
        "Initializing {} v{}.",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );

    let token = std::env::var("DISCORD_TOKEN").expect("Expected DISCORD_TOKEN in environment.");
    let db_uri = std::env::var("DATABASE_URL").expect("Expected DATABASE_URL in environment.");
    let storage_dir = std::env::var("STORAGE_DIR").expect("Expected STORAGE_DIR in environment.");

    let db = PgPool::connect(&db_uri)
        .await
        .expect("Couldn't connect to database.");
    log::debug!("Connected to database.");

    sqlx::migrate!()
        .run(&db)
        .await
        .expect("Error while migrating database.");
    log::debug!("Database migrated.");

    let storage_dir = PathBuf::from(&storage_dir);
    tokio::fs::create_dir_all(&storage_dir)
        .await
        .unwrap_or_else(|_| panic!("Couldn't create storage directory: {storage_dir:?}"));
    log::debug!("Storage directory at {storage_dir:?} created.");

    let mut commands = vec![register(), help(), invite(), about()];
    commands.extend(Vec::from(COMMANDS.map(|f| f())));
    log::debug!(
        "Found commands: {:?}.",
        commands.iter().map(|c| c.name).collect::<Vec<&str>>()
    );

    let options = poise::FrameworkOptions {
        commands,
        prefix_options: poise::PrefixFrameworkOptions {
            mention_as_prefix: true,
            ..Default::default()
        },
        on_error: |error| Box::pin(on_error(error)),
        listener: move |_ctx, event, _framework, _data| {
            Box::pin(async move {
                if let poise::Event::Ready { .. } = event {
                    log::info!("Ready.")
                };
                Ok(())
            })
        },
        ..Default::default()
    };

    let framework = poise::Framework::build()
        .token(token)
        .user_data_setup(move |_ctx, _ready, _framework| {
            Box::pin(async move { Ok(Data::new(db, storage_dir)) })
        })
        .options(options)
        .client_settings(songbird::register)
        .build()
        .await
        .expect("Couldn't build command framework.");

    framework
        .start()
        .await
        .expect("Couldn't start command framework.");
}

fn main() {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            #[cfg(target_os = "linux")]
            {
                use tokio::signal::unix::{signal, SignalKind};
                let mut sigterm =
                    signal(SignalKind::terminate()).expect("Failed to listen to SIGTERM.");
                let mut sigint =
                    signal(SignalKind::interrupt()).expect("Failed to listen to SIGINT");
                tokio::select! {
                    _ = do_main() => log::error!("Bot crashed!"),
                    _ = sigterm.recv() => log::info!("Received SIGTERM. Shutting down..."),
                    _ = sigint.recv() => log::info!("Received SIGINT. Shutting down...")
                }
            }
            #[cfg(target_os = "windows")]
            {
                use tokio::signal::windows::ctrl_c;
                let mut ctrl_c = ctrl_c().expect("Failed to listen to CTRL-C");
                tokio::select! {
                    _ = do_main() => log::error!("Bot crashed!"),
                    _ = ctrl_c.recv() => log::info!("Received CTRL-C. Shutting down...")
                }
            }
        });
    log::info!("Goodbye.");
}
