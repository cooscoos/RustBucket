#[macro_use] extern crate rocket;
#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_migrations;
#[macro_use] extern crate rocket_sync_db_pools;


mod stats;
mod linux_logs;
use std::thread;

use linux_logs::readings;

use rocket::http::hyper::server::Server;
use rocket::{Rocket, Build};
use rocket::fairing::AdHoc;
use rocket::request::FlashMessage;
use rocket::response::{Flash, Redirect};
use rocket::serde::Serialize;
use rocket::fs::{FileServer, relative};

use rocket::response::stream::{Event, EventStream};
use rocket::tokio::time::{self, Duration};

use rocket::futures::stream::Stream;
use rocket::response::stream::stream;
use rocket::futures::stream::{self, StreamExt};
use rocket::Shutdown;
use rocket::tokio::select;
use rocket_dyn_templates::Template;
use rocket::State;
use rocket::form::Form;
use rocket::tokio::sync::broadcast::{channel, Sender, error::RecvError};

//use futures::channel::oneshot;





use tokio::sync::mpsc;

/// Receive a message from a form submission and broadcast it to any receivers.
#[post("/message", data = "<form>")]
async fn post(form: Form<String>, queue: &State<Sender<String>> ) {
    let _res = queue.send("Sent a message 2 u".to_string());
}

#[post("/todo")]
async fn make_stream(conn: DbConn, queue: &State<Sender<String>>, mut shutdown: Shutdown) {


    let mut rx = queue.subscribe();
    // it's because this is a queue...
    let mut interval = time::interval(Duration::from_secs(1));
    loop {
        select! {
            _ = interval.tick() => {
                println!("yes");

                let time = readings::get_time_string();
                let cputemps = readings::read_temp().unwrap();
                let memory = readings::read_memory().unwrap();
            
                let log = Log { localdate: time, cpu_temp: cputemps, memuse: memory.used, mem: memory.available };
            
                if let Err(e) = CompStat::insert(log, &conn).await {
                    error_!("Database insertion error: {}", e);
                    //return Flash::error(Redirect::to("/"), "Logs could not be inserted due an internal error.")
                }
            },
            msg = rx.recv() => match msg {
                Ok(_) => break,
                Err(_)=> {info!("Error. Hit shutdown to stop.");continue;},
            },
            _ = &mut shutdown => break,
        }
    }
}



#[post("/shutdown")]
fn shutdown(shutdown: Shutdown) -> &'static str {
    shutdown.notify();
    "Shutting down..."
}

use crate::stats::{CompStat, Log};

#[database("sqlite_database")]
pub struct DbConn(diesel::SqliteConnection);

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
struct Context {
    flash: Option<(String, String)>,
    tasks: Vec<CompStat>
}

impl Context {
    pub async fn err<M: std::fmt::Display>(conn: &DbConn, msg: M) -> Context {
        Context {
            flash: Some(("error".into(), msg.to_string())),
            tasks: CompStat::all(conn).await.unwrap_or_default()
        }
    }

    pub async fn raw(conn: &DbConn, flash: Option<(String, String)>) -> Context {
        match CompStat::all(conn).await {
            Ok(tasks) => Context { flash, tasks },
            Err(e) => {
                error_!("DB Task::all() error: {}", e);
                Context {
                    flash: Some(("error".into(), "Fail to access database.".into())),
                    tasks: vec![]
                }
            }
        }
    }
}

#[post("/showlogs")]
async fn new(conn: DbConn) -> Flash<Redirect> {

    //return db items
    return Flash::success(Redirect::to("/"), "Log successfully added.")


}



#[delete("/")]
async fn delete(conn: DbConn) -> Flash<Redirect> {
    if let Err(e) = CompStat::delete_all(&conn).await {
        error_!("Database deletion error: {}", e);
        Flash::error(Redirect::to("/"), "Logs could not be inserted due an internal error.")
    } else {
        Flash::success(Redirect::to("/"), "Logs successfully deleted.")
    }
}



#[get("/")]
async fn index(flash: Option<FlashMessage<'_>>, conn: DbConn) -> Template {

    let flash = flash.map(FlashMessage::into_inner);
    Template::render("index", Context::raw(&conn, flash).await)

 }

async fn run_migrations(rocket: Rocket<Build>) -> Rocket<Build> {
    // This macro from `diesel_migrations` defines an `embedded_migrations`
    // module containing a function named `run`. This allows the example to be
    // run and tested without any outside setup of the database.
    embed_migrations!();

    let conn = DbConn::get_one(&rocket).await.expect("database connection");
    conn.run(|c| embedded_migrations::run(c)).await.expect("can run migrations");

    rocket
}

#[launch]
fn rocket() -> Rocket<Build> {
    rocket::build()
        .attach(DbConn::fairing())
        .attach(Template::fairing())
        .attach(AdHoc::on_ignite("Run Migrations", run_migrations))
        .manage(channel::<String>(1024).0)
        .mount("/", FileServer::from(relative!("static")))
        .mount("/", routes![index, shutdown,post,make_stream ])
        .mount("/todo", routes![new, delete])

}
