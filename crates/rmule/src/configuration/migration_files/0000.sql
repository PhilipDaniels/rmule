-- Create the version table.

-- The version table is used to manage database migrations.
CREATE TABLE version(version INT);

INSERT INTO version(version) VALUES (0);
