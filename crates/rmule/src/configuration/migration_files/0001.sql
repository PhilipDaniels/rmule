-- Create the settings table.

-- Stores the rMule settings. The equivalent of the amule.conf file.
CREATE TABLE settings
    (
    created TEXT NOT NULL,
    updated TEXT NOT NULL,
    -- Nickname for use on the network.
    nick_name TEXT NOT NULL,
    -- Default directory into which to place downloads if not set when the download is created.
    default_downloads_directory TEXT NOT NULL,
    -- Flag: whether to update the server list automatically when the program starts.
    auto_update_server_list INT NOT NULL
    );
