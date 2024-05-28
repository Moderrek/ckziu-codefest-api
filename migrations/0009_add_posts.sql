create table posts
(
    id serial not null primary key,
    owner_id   uuid                     not null,
    content    varchar                  not null,
    created_at timestamp with time zone not null default (now()),
    updated_at timestamp with time zone not null default (now())
);

create unique index posts_id_idx on posts (id);
create index posts_owner_id_idx on posts (owner_id);
