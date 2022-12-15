-- Create the settings table.
CREATE TABLE settings
    (
    created TEXT NOT NULL,
    updated TEXT NOT NULL,
    nick_name TEXT NOT NULL,
    default_downloads_directory TEXT NOT NULL,
    auto_update_server_list INT NOT NULL
    );
