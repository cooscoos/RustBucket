# RustBucket

This Rust program will:

1. grab one log per second of cpu temperature and memory usage from a host system running Linux (tested on Debian derivatives Rasperry Pi OS and Pop!_OS);
2. store logs in an sqlite database on the host system, and;
3. launch a webpage for you to start, stop, see or delete logs, or shut down the program remotely.

You can set it up so that you can access the webpage on any device in the same network.

For a Golang implementation of this, see [GoBucket](https://github.com/cooscoos/GoBucket).

![snapshot of the app](/snapshot.png "snapshot of the app")

## To use 

On the host system (the one you want to monitor)

1. Install sqlite3:
`sudo apt install libsqlite3-dev`

2. Download this here RustBucket github repo.

3. Optional: By default this program will launch a web page at http://localhost:8000. If you want to access the web page from another device on the same network then uncomment and edit the following two lines in Rocket.toml to match the address and port you'd like to access the page from:
`address = "0.0.0.0"`
`port = "8000"`

4. Get Rust and then compile this repo with `cargo run` or `cargo build`.

5. Go to http://localhost:8000/ or the http://address:port from step 3.
