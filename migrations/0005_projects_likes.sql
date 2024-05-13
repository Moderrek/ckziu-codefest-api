create table projects_likes
(
    project_id uuid                     not null,
    user_id    uuid                     not null,
    liked_at   timestamp with time zone not null default (now())
);

create unique index projects_likes_project_id_idx on projects_likes (project_id);
create unique index projects_likes_user_id_idx on projects_likes (user_id);
