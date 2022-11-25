-- Create the settings table and populate initial values.
CREATE TABLE settings
    (
    created TIMESTAMP NOT NULL DEFAULT (STRFTIME('%Y-%m-%d %H:%M:%f', 'NOW')),
    updated TIMESTAMP NOT NULL DEFAULT (STRFTIME('%Y-%m-%d %H:%M:%f', 'NOW')),
    nick_name TEXT NOT NULL
    );

INSERT INTO settings(nick_name) VALUES ('http://www.rMule.org');
