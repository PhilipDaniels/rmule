-- Create the server table.
-- This is the rmule equivalent of the "server.met" filesetting from emule.
CREATE TABLE server
    (
    id INTEGER PRIMARY KEY,
    created TIMESTAMP NOT NULL DEFAULT (STRFTIME('%Y-%m-%d %H:%M:%f', 'NOW')),
    updated TIMESTAMP NOT NULL DEFAULT (STRFTIME('%Y-%m-%d %H:%M:%f', 'NOW')),
    source TEXT NOT NULL,
    active INTEGER NOT NULL,
    -- The above fields are things introduced by rMule.
    -- The following are the 'business' fields, i.e. those found in server.met.
    ip_addr TEXT NOT NULL UNIQUE,
    port INTEGER NOT NULL,
    name TEXT,
    description TEXT,
    user_count INTEGER,
    low_id_user_count INTEGER,
    max_user_count INTEGER,
    ping_ms INTEGER,
    file_count INTEGER,
    soft_file_limit INTEGER,
    hard_file_limit INTEGER,
    udp_flags INTEGER,
    version TEXT,
    last_ping_time TIMESTAMP,
    udp_key TEXT,
    udp_key_ip_addr TEXT,
    tcp_obfuscation_port INTEGER,
    udp_obfuscation_port INTEGER,
    dns_name TEXT,
    priority INTEGER,
    aux_ports_list TEXT,
    fail_count INTEGER
    );
