use crate::config::Config;
use crate::process::{Process, Status};
use mysql::prelude::Queryable;
use mysql::Pool;

pub struct DbManager {
    pool: Pool,
}

impl DbManager {
    pub fn new(config: &Config) -> Result<DbManager, String> {
        let url = &config.url;
        let pool = match Pool::new(url) {
            Ok(v) => v,
            Err(e) => return Err(e.to_string()),
        };
        Ok(DbManager { pool })
    }

    pub fn get_process(&self, hostname: &str, pid: u32) -> Result<Process, String> {
        let mut conn = self.pool.get_conn().map_err(|e| e.to_string())?;
        let wait_process = conn
            .query_map(
                format!(
                    "SELECT * FROM process_list WHERE hostname = \"{}\" AND pid = {}",
                    hostname, pid
                ),
                |(id, hostname, pid, status)| Process {
                    id,
                    hostname,
                    pid,
                    status: Status::from_u32(status),
                },
            )
            .map_err(|e| e.to_string())?;

        if wait_process.len() > 1 {
            Err("Should only have one such process!".to_string())
        } else if wait_process.len() == 0 {
            Err("Process does not exist".to_string())
        } else {
            let wait_process = wait_process.get(0).unwrap().clone();
            Ok(wait_process)
        }
    }

    pub fn create_process(&self, hostname: &str, pid: u32) -> Result<(), String> {
        let mut conn = self.pool.get_conn().map_err(|e| e.to_string())?;
        conn.exec_drop(
            "INSERT INTO process_list (hostname, pid, status) VALUES (?, ?, ?)",
            (hostname, pid, 0),
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn update_process_state(
        &self,
        hostname: &str,
        pid: u32,
        status: Status,
    ) -> Result<(), String> {
        let mut conn = self.pool.get_conn().map_err(|e| e.to_string())?;
        conn.exec_drop(
            "UPDATE process_list SET status = ? WHERE hostname = ? AND pid = ?",
            (status as u32, hostname, pid),
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }
}
