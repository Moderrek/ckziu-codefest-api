create table auth
(
    mail     varchar not null unique,
    id       uuid    not null unique,
    password varchar not null
);

create unique index auth_mail_idx on auth (mail);
