
amule.cpp/364
  This is CamuleApp::OnInit, where most initialization seems to occur
  /424 Something about SetOSFiles, skipped
  /447 Something about SetECPass, skipped
  /455, root user check, skipped for now
  /503 start to create data structures: CStatistics, CClientList, CFriendList, CSearchList etc.

UI TODO
=======
[ ] How to stop "Server Name" from wrapping in grid header
[ ] For each From<X> Into WidgetText, add an Option<> blanket impl.
[ ] Display the log
[ ] Icon buttons for the toolbar
[ ] Spacing/striping in the server list

Small Things TODO
=================
[ ] Fix the exit code in parse_args
[ ] Consider using r2d2-sqlite for connection pooling. Remove the
    stored connection in the ConfigurationManager.
    [ ] Created a pooled connection type which can be used as a param
        so that a connection can be passed in, but generated if None
        is passed - enables connection reuse.
[ ] Signal handling

Big Things TODO
===============
[ ] Connect to server
[ ] Run a search
[ ] Allow multiple temp dirs to point to the same location
[ ] Create DbCollection and DbEntity traits (load_all, delete_all, insert, update etc.)

Future Ideas
============
[ ] Delete a temp db (sqlite file) if it becomes empty
[ ] Run as a daemon (PID file needed)
[ ] Caching.
  [ ] X minutes.
  [ ] If an entire file arrives without needing flushing to db we can write the whole
      thing to the destination without needing to write to db first.
  [ ] Allow files to be held for preview without actually writing them out to disk first

Egui Notes
==========
- shape.rs: PathShape and Shape: used to create polygons. See eg
  Shape::convex_polygon() and visual_bounding_rect().
- style.rs: see Style struct. visuals is colors. spacing is Spacing...
- painter.rs: used for actually drawing. See in particular fade_to_color "used for grayed
  out interfaces". It also has some primitive painting functions.
- ui.rs: has an set_enabled() function.

Split
=====
# rmule
- constructs the Engine

# rmuled
- has no code for now

# rmulegui
- sends commands such as "init"
- receives events such as "init complete"
- will eventually need to add arg to specify rmuled's  ip:port
- gui DOES require single instance, but can have multiple windows


# Main crates used
- [rusqlite](https://crates.io/crates/rusqlite) rMule stores its configuration in a SQLite database, and the temporary
  download files are stored in one or more SQLite databases before being finally
  written to their completed, physical OS files.
- [tokio](https://crates.io/crates/tokio) is used for network and disk IO, and its channels are used for communication
  between the various subsystems of rMule.
- [tracing](https://crates.io/crates/tracing), a.k.a. "tokio-tracing", is used for instrumentation.
- [anyhow](https://crates.io/crates/anyhow) is used for error handling (really error type *conversion*) throughout.
- [pico-args](https://crates.io/crates/pico-args) is used to parse the command line arguments. They're simple, and there is no need for something as heavyweight as [clap](https://crates.io/crates/clap).
- [egui](https://crates.io/crates/egui) and [eframe](https://crates.io/crates/eframe) are used for the UI.
- 

rMule avoids bringing in crates where possible. For example, I don't use
[diesel](https://crates.io/crates/diesel) for SQL access, and rMule has its own database
migration system in less than 100 lines of code.


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
