-- Add up migration script here
create or replace function set_updated_at() returns trigger as
$$
begin
    new.updated_at = current_timestamp;
    return new;
end;
$$ language plpgsql;


create table guilds
(
    id         bigint primary key,
    created_at timestamp with time zone not null default current_timestamp,
    updated_at timestamp with time zone not null default current_timestamp,
    deleted_at timestamp with time zone          default null,

    unique (id, deleted_at)
);

create index idx_guilds_deleted_at on guilds (deleted_at asc);

create trigger set_guilds_updated_at
    before update
    on guilds
    for each row
execute procedure set_updated_at();


create table sounds
(
    id          int generated always as identity primary key,
    created_at  timestamp with time zone not null default current_timestamp,
    updated_at  timestamp with time zone not null default current_timestamp,
    deleted_at  timestamp with time zone          default null,

    guild_id    bigint                   not null,
    name        text                     not null,
    source      text                     not null,
    uploader_id bigint                   not null,
    length      int                      not null,

    foreign key (guild_id) references guilds (id) on delete cascade,
    unique (guild_id, name, deleted_at)
);

create index idx_sounds_deleted_at on sounds (deleted_at asc);

create trigger set_sounds_updated_at
    before update
    on sounds
    for each row
execute procedure set_updated_at();

create table playbacks
(
    id         int generated always as identity primary key,
    created_at timestamp with time zone not null default current_timestamp,
    updated_at timestamp with time zone not null default current_timestamp,
    deleted_at timestamp with time zone          default null,

    stopped_at timestamp with time zone          default null,
    sound_id   integer                  not null,
    player_id  bigint                   not null,
    stopper_id bigint                            default null,

    foreign key (sound_id) references sounds (id) on delete cascade
);

create index idx_playbacks_deleted_at on playbacks (deleted_at asc);

create trigger set_playbacks_updated_at
    before update
    on playbacks
    for each row
execute procedure set_updated_at();
