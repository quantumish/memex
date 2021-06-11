pub enum Query {
	EDIT,
	GET,
	ADD,	
	DEL,
}

pub enum Entity {
	Block(Vec<String>, String),
	Tag(String),
	Project(String),
}

pub struct Request {
	pub query: Query,
	pub entity: Entity,
}

