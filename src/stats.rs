// Uses diesel to manage an sqlite database of system logs
use diesel::{self, prelude::*, result::QueryResult};
use rocket::serde::Serialize;

mod schema {
    table! {
        compstats {
            id -> Nullable<Integer>,
            localdate -> Text,
            cpu_temp -> Integer,
            memuse -> Float,
            mem -> Float,
        }
    }
}

use self::schema::compstats;
use self::schema::compstats::dsl::compstats as all_compstats;

use crate::DbConn;

#[derive(Serialize, Queryable, Insertable, Debug, Clone)]
#[serde(crate = "rocket::serde")]
#[table_name = "compstats"]
pub struct CompStat {
    pub id: Option<i32>,
    pub localdate: String,
    pub cpu_temp: i32,
    pub memuse: f32,
    pub mem: f32,
}

#[derive(Debug, FromForm)]
pub struct Log {
    pub localdate: String,
    pub cpu_temp: i32,
    pub memuse: f32,
    pub mem: f32,
}

impl CompStat {
    // Return the entire database
    pub async fn all(conn: &DbConn) -> QueryResult<Vec<CompStat>> {
        conn.run(|c| {
            all_compstats
                .order(compstats::id.desc())
                .load::<CompStat>(c)
        })
        .await
    }

    // Return the most recent num_limit results from the database
    pub async fn selection(conn: &DbConn, num_limit: i64) -> QueryResult<Vec<CompStat>> {

        // Try get the most recent records
        match conn.run(move |c| {
            all_compstats
                .order(compstats::id.desc())
                .limit(num_limit)
                .load::<CompStat>(c)
        })
        .await {
            Ok(val) => Ok(val),
            Err(_) => {
                // Errors occur because the user requested more logs than are available, or the request was numlimit=-1
                // If that's the case then just return all logs in the db
                CompStat::all(conn).await
            }
        }
    }

    /// Returns the number of affected rows: 1.
    pub async fn insert(log: Log, conn: &DbConn) -> QueryResult<usize> {
        conn.run(move |c| {
            let t = CompStat {
                id: None,
                localdate: log.localdate,
                cpu_temp: log.cpu_temp,
                memuse: log.memuse,
                mem: log.mem,
            };
            diesel::insert_into(compstats::table).values(&t).execute(c)
        })
        .await
    }

    /// Returns the number of affected rows: 1.
    /// Not really used
    pub async fn delete_with_id(id: i32, conn: &DbConn) -> QueryResult<usize> {
        conn.run(move |c| diesel::delete(all_compstats.find(id)).execute(c))
            .await
    }

    /// Returns the number of affected rows.
    pub async fn delete_all(conn: &DbConn) -> QueryResult<usize> {
        conn.run(|c| diesel::delete(all_compstats).execute(c)).await
    }
}

/// Need to make an Html display version of compstat that outputs strings because
/// I don't know enough html to format float precision of passed variables
/// ¯\_ (ツ)_/¯
#[derive(Serialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct HtmlCompStat {
    pub localdate: String,
    pub localtime: String,
    pub cpu_temp: String,
    pub memuse: String,
    pub mem: String,
}

// Take the CompStat from the database and format the precision for the webpage
impl HtmlCompStat {
    pub fn default(raw_compstat: &CompStat) -> Self {
        HtmlCompStat {
            localdate: raw_compstat.localdate[0..10].to_string(),
            localtime: raw_compstat.localdate[11..19].to_string(),
            cpu_temp: format!("{}", raw_compstat.cpu_temp),
            memuse: format!("{:.2}", raw_compstat.memuse),
            mem: format!("{:.2}", raw_compstat.mem),
        }
    }
}
