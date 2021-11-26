pub use chrono::{Duration, DateTime, Local};
use serde::{Serialize, Deserialize};

pub const MAX_NAME: usize = 64;
pub const MAX_ATTR_NAME: usize = 16;

#[derive(Serialize, Deserialize)]
pub enum Query {
    GET(Specifier),
    ADD(Entity),
    LOG(Range),
    // DEL,
}

#[derive(Serialize, Deserialize)]
pub enum Range {
    TimeRange(DateTime<Local>, DateTime<Local>),
    RelativeRange(usize, usize),
    Term(Term),
}

#[derive(Serialize, Deserialize)]
pub enum Term {
    Today,
    Yesterday,
    Week,
    Month,
    Year,
    All,
}

#[derive(Serialize, Deserialize)]
pub enum Specifier {
    Relative(usize),
    Id(String),
}

#[derive(Serialize, Deserialize)]
pub enum Entity {
    Block(String, String),
    Tag(String),
    Project(String),
}

#[derive(Serialize, Deserialize)]
pub struct Request {
    pub query: Query,
}
