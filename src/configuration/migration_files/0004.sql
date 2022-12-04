-- Create the server table.
-- This is the rmule equivalent of the "server.met" filesetting from emule.
CREATE TABLE server
    (
    id INTEGER PRIMARY KEY,
    created TIMESTAMP NOT NULL DEFAULT (STRFTIME('%Y-%m-%d %H:%M:%f', 'NOW')),
    updated TIMESTAMP NOT NULL DEFAULT (STRFTIME('%Y-%m-%d %H:%M:%f', 'NOW')),
    ip TEXT NOT NULL,
    port INTEGER NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    ping INTEGER,
    fail_count INTEGER,
    priority INTEGER NOT NULL,
    dns TEXT NOT NULL,
    max_users INTEGER,
    soft_files INTEGER NOT NULL,
    hard_files INTEGER NOT NULL,
    last_ping_time TIMESTAMP,
    version TEXT,
    udp_flags INTEGER,
    aux_ports_list TEXT,
    users INTEGER,
    files INTEGER,
    source TEXT NOT NULL,
    active INTEGER NOT NULL
    );
