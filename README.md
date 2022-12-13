amule.cpp/364
  This is CamuleApp::OnInit, where most initialization seems to occur
  /424 Something about SetOSFiles, skipped
  /447 Something about SetECPass, skipped
  /455, root user check, skipped for now
  /503 start to create data structures: CStatistics, CClientList, CFriendList, CSearchList etc.

[ ] Created, updated and last_ping_time: need to decide on data type

[ ] Run as a daemon (PID file needed)
[ ] Caching.
  [ ] X minutes.
  [ ] If an entire file arrives without needing flushing to db we can write the whole
      thing to the destination without needing to write to db first.

# Timeline
1. Do not commence network calls until all init is complete.
2. Load Configuration Manager from db.
3. Emit ConfigurationLoaded.
4. Begin auto-update of server list.
5. Emit ConfigurationFinished.

# Main crates used
- [rusqlite](https://crates.io/crates/rusqlite) rMule stores its configuration in a SQLite database, and the temporary
  download files are stored in one or more SQLLite databases before being finally
  written to their completed, physical OS files.
- [tokio](https://crates.io/crates/tokio) is used for network and disk IO, and its channels are used for communication
  between the various subsystems of rMule.
- [tracing](https://crates.io/crates/tracing), a.k.a. "tokio-tracing", is used for instrumentation.
- [anyhow](https://crates.io/crates/anyhow) is used for error handling (really error type *conversion*) throughout.
- [pico-args](https://crates.io/crates/pico-args) is used to parse the command line arguments. They're simple, and there is no need for something as heavyweight as [clap](https://crates.io/crates/clap).


rMule avoids bringing in crates where possible. For example, I don't use
[diesel](https://crates.io/crates/diesel) for SQL access, and rMule has its own database migration system in less than 100 lines of code.


https://crates.io/crates/tokio-console

# Differences vs aMule/eMule

## Backend
- rMule stores its configuration and downloading files in SQLite databases rather than
  disk files. The files are cross-platform and easily moved to other computers.
- rMule is written in Rust and can run on Linux, Windows and MacOS with one codebase.
- Multiple temporary directories may be configured, which helps with distributing disk IO
  over multiple physical disks in the case that you don't have a RAID system.
- The "download" folder may be specified when enqueing a download.

## GUI
- Multiple UI windows may be opened to monitor the same running rMule.
- The 'progress bar' for a download shows 'grouped' progress rather than
  'by chunk'.
