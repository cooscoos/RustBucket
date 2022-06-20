// External crates
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate rocket_sync_db_pools;

use rocket::fairing::AdHoc;
use rocket::fs::{relative, FileServer};
use rocket::request::FlashMessage;
use rocket::response::{Flash, Redirect};
use rocket::serde::Serialize;
use rocket::tokio::select;
use rocket::tokio::sync::broadcast::{channel, Sender};
use rocket::tokio::time::{self, Duration};
use rocket::{Build, Rocket, Shutdown, State};
use rocket_dyn_templates::Template;

// Internal modules
mod linux_logs;
mod stats;
use crate::stats::{CompStat, Log};
use linux_logs::readings;

// Set up sqlite database
#[database("sqlite_database")]
pub struct DbConn(diesel::SqliteConnection);

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
struct Context {
    flash: Option<(String, String)>,
    logged_stats: Vec<CompStat>,
}

impl Context {
    pub async fn err<M: std::fmt::Display>(conn: &DbConn, msg: M) -> Context {
        Context {
            flash: Some(("error".into(), msg.to_string())),
            logged_stats: CompStat::all(conn).await.unwrap_or_default(),
        }
    }

    pub async fn raw(conn: &DbConn, flash: Option<(String, String)>) -> Context {
        match CompStat::all(conn).await {
            Ok(stats) => Context {
                flash,
                logged_stats: stats,
            },
            Err(e) => {
                error_!("DB Task::all() error: {}", e);
                Context {
                    flash: Some(("error".into(), "Failed to access database.".into())),
                    logged_stats: vec![],
                }
            }
        }
    }

}

// Broadcast an empty message, this will break out of the logging loop in async fn start_logs
#[post("/logs/stop")]
async fn stop_logs(queue: &State<Sender<()>>) -> Flash<Redirect> {
    let _res = queue.send(());
    Flash::success(Redirect::to("/"), "Logs successfully stopped.")
}

#[post("/shutdown")]
fn shutdown(shutdown: Shutdown) -> &'static str {
    shutdown.notify();
    "Shut down. Restart process on host to resume."
}

// Start logging host stats to sqlite
#[post("/logs/start")]
async fn start_logs(conn: DbConn, queue: &State<Sender<()>>, mut shutdown: Shutdown) {
    let mut rx = queue.subscribe();
    let mut interval = time::interval(Duration::from_secs(1));
    loop {
        select! {                               // Select whichever happens first
            _ = interval.tick() => {            // The next second happens,
                println!("Log acquired.");

                let time = readings::get_time_string();
                let cputemps = readings::read_temp().unwrap();
                let memory = readings::read_memory().unwrap();

                let log = Log { localdate: time, cpu_temp: cputemps, memuse: memory.used, mem: memory.available };

                if let Err(e) = CompStat::insert(log, &conn).await {
                    error_!("Database insertion error: {}", e);
                }
            },
            msg = rx.recv() => match msg {      // queue recieves a message (set by async fn stop_logs)
                Ok(_) => break,
                Err(_)=> {info!("Error. Hit shutdown to stop.");continue;},
            },
            _ = &mut shutdown => break,         // receive notification to shutdown
        }
    }
}

#[post("/logs/show")]
async fn show_logs(conn: DbConn) -> Flash<Redirect> {
    // todo: return db items
    // At the moment flash redirect is showing the items in the db ...?
    return Flash::success(Redirect::to("/"), "Log successfully added.");
}

#[delete("/logs/delete")]
async fn delete_logs(conn: DbConn) -> Flash<Redirect> {
    if let Err(e) = CompStat::delete_all(&conn).await {
        error_!("Database deletion error: {}", e);
        Flash::error(
            Redirect::to("/"),
            "Logs could not be inserted due an internal error.",
        )
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
    conn.run(|c| embedded_migrations::run(c))
        .await
        .expect("can't run migrations");

    rocket
}

#[launch]
fn rocket() -> Rocket<Build> {
    rocket::build()
        .attach(DbConn::fairing())
        .attach(Template::fairing())
        .attach(AdHoc::on_ignite("Run Migrations", run_migrations))
        .manage(channel::<()>(1).0)
        .mount("/", FileServer::from(relative!("static")))
        .mount(
            "/",
            routes![
                index,
                shutdown,
                stop_logs,
                start_logs,
                show_logs,
                delete_logs
            ],
        )
}