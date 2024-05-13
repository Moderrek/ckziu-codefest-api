create table projects_data
(
    id          uuid    not null primary key,
    overview    varchar not null,
    github_url  varchar,
    website_url varchar
);

create unique index projects_data_id_idx on projects_data (id);
