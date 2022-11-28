-- Create the temp_directory table.
-- This is the rmule equivalent of the "temp directory" setting from emule.
-- rmule supports multiple temp directories, which can help with
-- spreading disk IO across multiple devices, in case you don't have a
-- RAID array.
CREATE TABLE temp_directory
    (
    created TIMESTAMP NOT NULL DEFAULT (STRFTIME('%Y-%m-%d %H:%M:%f', 'NOW')),
    updated TIMESTAMP NOT NULL DEFAULT (STRFTIME('%Y-%m-%d %H:%M:%f', 'NOW')),
    directory TEXT NOT NULL PRIMARY KEY
    );

-- Will be normalized to the config_dir at run time.
INSERT INTO temp_directory(directory) VALUES ('temp');
