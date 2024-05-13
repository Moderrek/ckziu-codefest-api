ALTER TABLE projects
    ADD tournament BOOLEAN NOT NULL
        CONSTRAINT tournament DEFAULT false;