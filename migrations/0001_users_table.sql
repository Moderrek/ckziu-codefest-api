create table users
(
    name         varchar                  not null unique,
    display_name varchar                  not null,
    id           uuid                     not null unique,
    bio          varchar,
    avatar       varchar,
    created_at   timestamp with time zone not null default (now()),
    updated_at   timestamp with time zone not null default (now()),
    flags        integer                  not null default (0)
);

create unique index users_uuid_idx on users (id);
create unique index users_name_idx on users (name);
