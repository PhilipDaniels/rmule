-- Create the settings table.
CREATE TABLE settings
    (
    created TIMESTAMP NOT NULL DEFAULT (STRFTIME('%Y-%m-%d %H:%M:%f', 'NOW')),
    updated TIMESTAMP NOT NULL DEFAULT (STRFTIME('%Y-%m-%d %H:%M:%f', 'NOW')),
    nick_name TEXT NOT NULL,
    default_completed_directory TEXT NOT NULL
    );

INSERT INTO settings
    (
    nick_name, default_completed_directory
    )
VALUES
    (
    'http://www.rMule.org', 'completed'
    );