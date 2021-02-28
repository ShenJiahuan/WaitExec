#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Status {
    PENDING = 0,
    RUNNING = 1,
    DONE = 2,
    FAILED = 3,
}

impl Status {
    pub fn from_u32(value: u32) -> Status {
        match value {
            0 => Status::PENDING,
            1 => Status::RUNNING,
            2 => Status::DONE,
            3 => Status::FAILED,
            _ => panic!("Unknown value: {}", value),
        }
    }
}

#[derive(Clone)]
pub struct Process {
    pub id: i32,
    pub hostname: String,
    pub pid: u32,
    pub status: Status,
}
