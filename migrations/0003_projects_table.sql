create table projects
(
    id           uuid                     not null primary key,
    name         varchar                  not null,
    display_name varchar                  not null,

    owner_id     uuid                     not null,

    private      bool                     not null default (false),
    thumbnail    varchar                           default (null),
    description  varchar                           default (null),

    likes        int                      not null default (0),

    created_at   timestamp with time zone not null default (now()),
    updated_at   timestamp with time zone not null default (now())
);

create unique index projects_id_idx on projects (id);
create index projects_name_idx on projects (name);
create index projects_owner_idx on projects (owner_id);
