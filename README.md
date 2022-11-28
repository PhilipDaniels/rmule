amule.cpp/364
  This is CamuleApp::OnInit, where most initialization seems to occur
  /424 Something about SetOSFiles, skipped
  /447 Something about SetECPass, skipped
  /455, root user check, skipped for now
  /503 start to create data structures: CStatistics, CClientList, CFriendList, CSearchList etc.


Where can we use multi-threading?

[ ] List of addresses before servers?
[ ] List of servers. See ServerList.cpp for operations.


[ ] Run as a daemon (PID file needed)
[ ] Load server.met? Also see code on line 592 to auto-update the list.
[ ] Load shared files (there seem to be two files?)
[ ] Use interior mutability for execute_in_independent_transaction? Or implement clone()?
    - All these database writes will ultimately be on a single thread.
[ ] Download and test server.met with nom


# Main crates used
- [rusqlite](https://crates.io/crates/rusqlite) rMule stores its configuration in a SQLite database, and the temporary
  download files are stored in one or more SQLLite databases before being finally
  written to their completed, physical OS files.
- [tokio](https://crates.io/crates/tokio) is used for network and disk IO, and its channels are used for communication
  between the various subsystems of rMule.
- [tracing](https://crates.io/crates/tracing), a.k.a. "tokio-tracing", is used for instrumentation.
- [anyhow](https://crates.io/crates/anyhow) is used for error handling (really error type *conversion*) throughout.
- [nom](https://crates.io/crates/nom) is used to parse legacy aMule/eMule file formats such as
    [server.met](http://wiki.amule.org/t/index.php?title=Server.met_file)
- [pico-args](https://crates.io/crates/pico-args) is used to parse the command line arguments. They're simple, and there is no need for something as heavyweight as [clap](https://crates.io/crates/clap).


https://crates.io/crates/tokio-console

# Differences vs aMule/eMule

- rMule stores its configuration and downloading files in SQLite databases rather than
  disk files. The files are cross-platform and easily moved to other computers.
- rMule is written in Rust and can run on Linux, Windows and MacOS with one codebase.
- Multiple temporary directories may be configured, which helps with distributing disk IO
  over multiple physical disks in the case that you don't have a RAID system.
- The "download" folder may be specified when enqueing a download.
- Multiple UI windows may be opened to monitor the same running rMule.
