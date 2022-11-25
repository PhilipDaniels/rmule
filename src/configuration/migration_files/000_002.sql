-- Create the address table and populate initial values.
-- This table holds a list of URL from which server.met files can
-- be downloaded. It is the equivalent of the addresses.dat file
-- in amule. http://wiki.amule.org/wiki/Addresses.dat_file
CREATE TABLE address
    (
    created TIMESTAMP NOT NULL DEFAULT (STRFTIME('%Y-%m-%d %H:%M:%f', 'NOW')),
    updated TIMESTAMP NOT NULL DEFAULT (STRFTIME('%Y-%m-%d %H:%M:%f', 'NOW')),
    url TEXT NOT NULL PRIMARY KEY,
    active INT NOT NULL
    );
