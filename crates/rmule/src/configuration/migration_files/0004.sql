-- Create the server table.

-- This is the rmule equivalent of the "server.met" filesetting from emule.
CREATE TABLE server
    (
    id INTEGER PRIMARY KEY,
    created TEXT NOT NULL,
    updated TEXT NOT NULL,
    -- Denormalized address from which this server was downloaded.
    source TEXT NOT NULL,
    -- Logical delete flag.
    active INTEGER NOT NULL,
    -- IP of the server in string form. Natural key of the table.
    ip_addr TEXT NOT NULL UNIQUE,
    -- TCP/IP port.
    port INTEGER NOT NULL,
    -- The friendly name of the server, e.g. "eMule Sunrise".
    name TEXT,
    -- Short description of the server.
    description TEXT,
    -- The number of users currently registered on the server.
    user_count INTEGER,
    -- The number of 'Low Id' users currently registered on the server.
    -- See http://wiki.amule.org/wiki/FAQ_eD2k-Kademlia#What_is_LowID_and_HighID.3F
    low_id_user_count INTEGER,
    -- Maximum number of users the server allows to simultaneously connect
    max_user_count INTEGER,
    -- Time (in ms) it takes to communicate with the server.
    ping_ms INTEGER,
    -- The number of files registered on the server.
    file_count INTEGER,
    -- Soft files is the minimum number of files you must share to not be penalized.
    soft_file_limit INTEGER,
    -- Hard files is the maximum number of files you must share to not be penalized.
    hard_file_limit INTEGER,
    -- What actions are supported via UDP.
    udp_flags INTEGER,
    -- Version and name of the software the server is running to support the ed2k network.
    version TEXT,
    -- The last time the server was pinged.
    last_ping_time TEXT,
    -- UNKNOWN
    udp_key INTEGER,
    -- UNKNOWN
    udp_key_ip_addr TEXT,
    -- UNKNOWN
    tcp_obfuscation_port INTEGER,
    -- UNKNOWN
    udp_obfuscation_port INTEGER,
    -- The DNS name of the server.
    dns_name TEXT,
    -- Server priority.
    priority INTEGER,
    -- Comma-separated list of extra ports the server supports.
    aux_ports_list TEXT,
    -- How many times connecting to the server failed (reset to 0 on success?)
    fail_count INTEGER
    );
