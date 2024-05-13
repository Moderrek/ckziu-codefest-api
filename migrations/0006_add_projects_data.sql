ALTER TABLE projects
    ADD content VARCHAR NOT NULL
        CONSTRAINT content_default DEFAULT '';
ALTER TABLE projects
    ADD github_url VARCHAR;
ALTER TABLE projects
    ADD website_url VARCHAR;
