pub const MAX_NAME: usize = 16;

#[derive(Debug)]
pub enum Query {
	EDIT,
	GET,
	ADD,	
	DEL,
}

#[derive(Debug)]
pub enum Entity {
	Block([u8; MAX_NAME], [u8; MAX_NAME]),
	Tag([u8; MAX_NAME]),
	Project([u8; MAX_NAME]),
}

#[derive(Debug)]
pub struct Request {
	pub query: Query,
	pub entity: Entity,
}



