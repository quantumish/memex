pub use chrono::{Duration, DateTime, Local};
pub const MAX_NAME: usize = 16;

pub enum Query {
	GET(Specifier),
	ADD(Entity),
	// DEL,
}

pub enum Specifier {
	Relative(usize),
	// Id(ID),
	Time(DateTime<Local>),
	TimeRange(DateTime<Local>, DateTime<Local>),
}

pub enum Entity {
	Block([u8; MAX_NAME], [u8; MAX_NAME]),
	Tag([u8; MAX_NAME]),
	Project([u8; MAX_NAME]),
}

pub struct Request {
	pub query: Query,
	// pub BULK: bool
}
