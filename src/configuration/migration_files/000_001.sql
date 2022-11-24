-- Create the settings table and populate initial values.
CREATE TABLE IF NOT EXISTS settings
    (
    downloaded_directory TEXT NOT NULL,
    nick_name TEXT NOT NULL
    );

INSERT INTO settings(downloaded_directory, nick_name) VALUES ('Downloaded', 'http://www.rMule.org');

