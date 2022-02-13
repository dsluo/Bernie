# Bernie

A(nother) soundboard bot for Discord.

Successor to [SoundBert](https://github.com/dsluo/SoundBert), which was written in Python using
[discord.py](https://github.com/Rapptz/discord.py). Since discord.py was discontinued, I decided
this would be a good opportunity to make something in Rust.

## Usage

```
Commands:
  /help        Send help.
  /invite      Get the invite for this bot.
  /about       Get info about the bot.
  /play        Play a sound in your current voice channel.
  /random      Play a random sound in your current voice channel.
  /stop        Stop the currently playing sound.
  /add         Add a new sound.
  /list        List all sounds on this server.
  /rename      Rename a sound.
  /remove      Delete a sound.
  /history     Show sound play history.
```

## Building

On Windows, this crate should build with just a `cargo build`.

On Linux, this crate needs `libopus-dev` and `pkg-config` to build its dependencies.

## Running

The bot uses PostgreSQL as its database. A `docker-compose.yml` is available to run both.

To run, the following environmental variables need to be set, in the actual environment, or in `.env`:

```shell
# For the postgres container
POSTGRES_DB=bernie
POSTGRES_USER=bernie
POSTGRES_PASSWORD=bernie1

# For bernie
DATABASE_URL=postgresql://${POSTGRES_USER}:${POSTGRES_PASSWORD}@localhost:5432/${POSTGRES_DB}
RUST_LOG="error,bernie=debug"
DISCORD_TOKEN=<discord token>
STORAGE_DIR=<sound file storage path>
```

If not running in Docker, Python3, ffmpeg, and yt-dlp need to be in the `PATH`, and on Linux `ca-certificates` needs to be installed (which it probably is).
