-- Create the address table.
-- This table holds a list of URL from which server.met files can
-- be downloaded. It is the equivalent of the addresses.dat file
-- in amule. http://wiki.amule.org/wiki/Addresses.dat_file
CREATE TABLE address
    (
    id INTEGER PRIMARY KEY,
    created TEXT NOT NULL,
    updated TEXT NOT NULL,
    url TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL,
    active INTEGER NOT NULL
    );
