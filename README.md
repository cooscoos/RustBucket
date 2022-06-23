
To do:

option to download the database
battery and network usage
docker



## RustBucket

This program will:

1. grab readings of cpu temperature, memory usage, etc from a host system running Linux (tested on Debian derivatives Rasperry Pi OS and Pop!_OS);
2. store those readings in an sqlite database on the host system, and;
3. launch a webpage for you to read the logs, delete or download the database, or shut down the host process remotely.

You can set it up so that you can access the webpage on any device in the same network.


### Docker install (Recommended)

A docker install allows you to containerise this process.

On the host system (the one you want to monitor)


### Manual install

On the host system (the one you want to monitor)

* Install sqlite3:
sudo apt install libsqlite3-dev

* Download this github repo.

* Optional: If you wish to access the web interface from another computer on the same network then uncomment and edit the following line in Rocket.toml
address = "0.0.0.0"
The address should match the IP address that your router has assigned to your host system. You can also modify the line:
port = "8000"
If you wish to change the port that the web interface is published on.

* Get rustup and then compile this repo with cargo run or cargo build.

* Go to the address shown in the terminal to see the webpage, either on the host system itself, or on another computer in the same network.
