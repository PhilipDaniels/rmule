-- Create the settings table and populate initial values.
CREATE TABLE IF NOT EXISTS settings
    (
    created TIMESTAMP NOT NULL DEFAULT (STRFTIME('%Y-%m-%d %H:%M:%f', 'NOW')),
    updated TIMESTAMP NOT NULL DEFAULT (STRFTIME('%Y-%m-%d %H:%M:%f', 'NOW')),
    downloaded_directory TEXT NOT NULL,
    nick_name TEXT NOT NULL
    );

INSERT INTO settings(downloaded_directory, nick_name) VALUES ('Downloaded', 'http://www.rMule.org');

