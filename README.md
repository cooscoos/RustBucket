# RustBucket

This Rust program will:

1. grab one log per second of cpu temperature and memory usage from a host system running Linux (tested on Debian derivatives Rasperry Pi OS and Pop!_OS);
2. store logs in an sqlite database on the host system, and;
3. launch a webpage for you to read or delete logs, or shut down the program remotely.

You can set it up so that you can access the webpage on any device in the same network.

## To use 

On the host system (the one you want to monitor)

* Install sqlite3:
sudo apt install libsqlite3-dev

* Download the current github repo.

* Optional: By default this process will launch a web page at http://localhost:8000. If you want to access the web page from another device on the same network then uncomment and edit the following lines in Rocket.toml
address = "0.0.0.0"
port = "8000"

* Get rustup and then compile this repo with cargo run or cargo build.

* Go to http://localhost:8000/ or the address:port you entered in step 3.