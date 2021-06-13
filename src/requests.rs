pub use chrono::{Duration, DateTime, Local};
pub const MAX_NAME: usize = 64;
pub const MAX_ATTR_NAME: usize = 16;

pub enum Query {
	GET(Specifier),
	ADD(Entity),
	LOG(Range, Fmt),
	// DEL,
}

pub enum Fmt {
	Oneline,
	Terse,
	Detail,
}

pub enum Range {
	TimeRange(DateTime<Local>, DateTime<Local>),
	RelativeRange(usize, usize),
	Term(Term),
}

pub enum Term {
	Today,
	Yesterday,
	Week,
	Month,
	Year,
	All,
}

pub enum Specifier {
	Relative(usize),
	Id([u8; MAX_ATTR_NAME]),
}

pub enum Entity {
	Block([u8; MAX_NAME], [u8; MAX_ATTR_NAME]),
	Tag([u8; MAX_ATTR_NAME]),
	Project([u8; MAX_ATTR_NAME]),
}

pub struct Request {
	pub query: Query,
	// pub BULK: bool
}
