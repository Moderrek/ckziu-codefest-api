create table auth
(
    name     varchar not null unique,
    mail     varchar not null unique,
    id       uuid    not null unique,
    password varchar not null
);

create unique index auth_uuid_idx on auth (id);
create unique index auth_mail_idx on auth (mail);
create unique index auth_name_idx on auth (name);
