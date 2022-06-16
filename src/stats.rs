use rocket::serde::Serialize;
use diesel::{self, result::QueryResult, prelude::*};

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
use self::schema::compstats::dsl::{compstats as all_compstats};

use crate::DbConn;

#[derive(Serialize, Queryable, Insertable, Debug, Clone)]
#[serde(crate = "rocket::serde")]
#[table_name="compstats"]
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
    pub async fn all(conn: &DbConn) -> QueryResult<Vec<CompStat>> {
        conn.run(|c| {
            all_compstats.order(compstats::id.desc()).load::<CompStat>(c)
        }).await
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
        }).await
    }

    /// Returns the number of affected rows: 1.
    /// Not really used
    pub async fn delete_with_id(id: i32, conn: &DbConn) -> QueryResult<usize> {
        conn.run(move |c| diesel::delete(all_compstats.find(id)).execute(c)).await
    }

    /// Returns the number of affected rows.
    pub async fn delete_all(conn: &DbConn) -> QueryResult<usize> {
        conn.run(|c| diesel::delete(all_compstats).execute(c)).await
    }
}
