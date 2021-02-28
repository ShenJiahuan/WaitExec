extern crate clap;
use clap::{App, Arg};
use std::fs;
use std::path::Path;
use std::process;
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;
use wait_exec::config::Config;
use wait_exec::db_manager::DbManager;
use wait_exec::process::Status::{DONE, FAILED, RUNNING};

fn get_hostname() -> Result<String, String> {
    let result = match hostname::get() {
        Ok(v) => match v.into_string() {
            Ok(v) => Ok(v),
            Err(_) => Err("Failed to convert from OsString to String".to_string()),
        },
        Err(e) => Err(e.to_string()),
    };
    result
}

fn get_pid() -> u32 {
    process::id()
}

fn wait(
    dbmgr: &DbManager,
    hostname: &str,
    pid: u32,
    cur_hostname: &str,
    cur_pid: u32,
) -> Result<(), String> {
    loop {
        let wait_process = dbmgr.get_process(hostname, pid)?;

        if wait_process.status == DONE {
            return Ok(());
        } else if wait_process.status == FAILED {
            dbmgr.update_process_state(cur_hostname, cur_pid, FAILED)?;
            return Err("Waiting process exited unexpectedly".to_string());
        }
        sleep(Duration::from_secs(10));
    }
}

fn execute(
    dbmgr: &DbManager,
    program: &str,
    cur_hostname: &str,
    cur_pid: u32,
) -> Result<(), String> {
    let mut program = Command::new("sh")
        .arg("-c")
        .arg(program)
        .spawn()
        .map_err(|e| e.to_string())?;

    dbmgr.update_process_state(cur_hostname, cur_pid, RUNNING)?;

    let status = match program.wait() {
        Ok(v) => v,
        Err(e) => {
            dbmgr.update_process_state(cur_hostname, cur_pid, FAILED)?;
            return Err(e.to_string());
        }
    };
    if status.success() {
        dbmgr.update_process_state(cur_hostname, cur_pid, DONE)?;
        Ok(())
    } else {
        dbmgr.update_process_state(cur_hostname, cur_pid, FAILED)?;
        if let Some(code) = status.code() {
            Err(format!("Program exited with code {}", code))
        } else {
            Err("Program was terminated by a signal".to_string())
        }
    }
}

fn main() {
    let matches = App::new("wait_exec")
        .version("1.0")
        .author("Jiahuan Shen <shenjiahuan@sjtu.edu.cn>")
        .about("Wait and execute program")
        .arg(
            Arg::with_name("config")
                .long("config")
                .help("Sets a custom config file")
                .required(true)
                .takes_value(true)
                .validator(|file| {
                    return if Path::new(&file).exists() {
                        Ok(())
                    } else {
                        Err(format!("{} does not exist!", file))
                    };
                }),
        )
        .arg(
            Arg::with_name("host")
                .long("host")
                .help("Host of program to wait until finished")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("pid")
                .long("pid")
                .help("Pid of program to wait until finished")
                .takes_value(true)
                .validator(|v| {
                    return if v.parse::<u32>().is_ok() {
                        Ok(())
                    } else {
                        Err(format!("Not a valid pid: {}", v))
                    };
                }),
        )
        .arg(
            Arg::with_name("instant")
                .long("instant")
                .help("Execute the program instantly"),
        )
        .arg(
            Arg::with_name("program")
                .long("program")
                .help("Program to execute")
                .required(true)
                .takes_value(true),
        )
        .get_matches();

    let hostname = matches.value_of("host");
    let pid = matches.value_of("pid");
    let instant = matches.is_present("instant");
    if (!instant && (hostname.is_none() || pid.is_none()))
        || instant && (hostname.is_some() || pid.is_some())
    {
        panic!("You must either execute the program instantly, or specify the hostname and pid of the waiting program");
    }

    let config = matches.value_of("config").unwrap();
    let config = match fs::read_to_string(config) {
        Ok(v) => v,
        Err(e) => panic!("Failed to read config {}, cause: \n{}", config, e),
    };
    let config: Config = toml::from_str(&config).expect("Invalid config");
    let dbmgr = match DbManager::new(&config) {
        Ok(v) => v,
        Err(e) => panic!("Failed to acquire db connection, cause: \n{}", e),
    };
    let program_to_execute = matches.value_of("program").unwrap();

    let cur_hostname = get_hostname().expect("Failed to get hostname");
    let cur_pid = get_pid();
    if let Err(e) = dbmgr.create_process(&cur_hostname, cur_pid) {
        panic!("Failed to create process, cause: \n{}", e);
    }

    if !instant {
        let hostname = hostname.unwrap();
        let pid: u32 = pid.unwrap().parse().unwrap();
        if let Err(e) = wait(&dbmgr, hostname, pid, &cur_hostname, cur_pid) {
            panic!("Wait failed, cause: \n{}", e);
        }
    }

    if let Err(e) = execute(&dbmgr, program_to_execute, &cur_hostname, cur_pid) {
        panic!("Execute failed, cause: \n{}", e);
    }
}
