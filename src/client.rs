use std::net::UdpSocket;
pub mod requests;
pub use crate::requests::*;
use std::thread;

fn pack(s: &str) -> [u8; MAX_NAME] {
	let chars: Vec<char> = s.chars().collect();
	let length = chars.len();
	if (length > MAX_NAME) {panic!("String too big to pack!");}
	let mut tmp: [u8; MAX_NAME] = [0; MAX_NAME];
	for i in 0..length {
		tmp[i] = chars[i] as u8;
	}
	tmp
}

fn main() {
	let socket;
	match UdpSocket::bind("127.0.0.1:34256") {
		Ok(x) => socket = x,
		Err(_) => panic!("AA"),
	}
	let req : Request = Request {
		query: Query::ADD,
		entity: Entity::Block(pack("abc"), pack("No."))
	};
	// Time to commit a gamer moment.
	unsafe {
		let buf = std::mem::transmute::<Request, [u8; std::mem::size_of::<Request>()]>(req);
		socket.send_to(&buf, "127.0.0.1:34254");
	}
	thread::sleep(std::time::Duration::from_secs(1));
	let req : Request = Request {
		query: Query::GET,
		entity: Entity::Block(pack(""), pack(""))
	};
	unsafe {
		let buf = std::mem::transmute::<Request, [u8; std::mem::size_of::<Request>()]>(req);
		socket.send_to(&buf, "127.0.0.1:34254");
	}
}

