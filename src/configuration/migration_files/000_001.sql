-- Create the settings table and populate some initial values.
CREATE TABLE IF NOT EXISTS settings(key TEXT PRIMARY KEY, value TEXT);

INSERT INTO settings(key, value) VALUES ('temp_directory', 'temp');
INSERT INTO settings(key, value) VALUES ('downloaded_directory', 'downloaded');
INSERT INTO settings(key, value) VALUES ('nick_name', 'http://www.rMule.org');

