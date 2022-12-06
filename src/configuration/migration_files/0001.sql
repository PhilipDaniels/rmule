-- Create the settings table.
CREATE TABLE settings
    (
    created TIMESTAMP NOT NULL DEFAULT (STRFTIME('%Y-%m-%d %H:%M:%f', 'NOW')),
    updated TIMESTAMP NOT NULL DEFAULT (STRFTIME('%Y-%m-%d %H:%M:%f', 'NOW')),
    nick_name TEXT NOT NULL,
    default_downloads_directory TEXT NOT NULL,
    auto_update_server_list INT NOT NULL
    );
