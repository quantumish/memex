use std::net::UdpSocket;
pub mod requests;
pub use crate::requests::*;

fn main() {
	let socket;
	match UdpSocket::bind("127.0.0.1:34256") {
		Ok(x) => socket = x,
		Err(_) => panic!("AA"),
	}
	let tags = vec![String::from("test"), String::from("test2")];
	let req : Request = Request {
		query: Query::ADD,
		entity: Entity::Block(tags, String::from("No."))
	};
	// Time to commit a gamer moment.
	unsafe {
		let buf = std::mem::transmute::<Request, [u8; std::mem::size_of::<Request>()]>(req);
		socket.send_to(&buf, "127.0.0.1:34254");
	}
}

