pub mod requests;
pub use crate::requests::*;

use std::thread;
use std::time;
use chrono::{Duration, DateTime, Local};
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{Read, Write};

fn unpack(s: [u8; MAX_NAME]) -> String {
	match std::str::from_utf8(&s) {
		Ok(x) => String::from(x),
		Err(_x) => panic!("Failed to unpack."),
	}
}

#[derive(Clone)]
struct Tag {
	name: String
}

#[derive(Clone)]
struct Project {
	name: String
}

#[derive(Clone)]
pub struct Block {
	name: String,
	start: DateTime<Local>,
	end: Option<DateTime<Local>>,
	tags: Vec<Tag>,
	project: Option<Project>,
}

impl Block {
	fn new() -> Block {
		Block {
			name: String::new(),
			start: Local::now(),
			end: None,
			tags: Vec::new(),
			project: None,
		}
	}

	fn get_duration(&self) -> Duration {
		match self.end {
			Some(x) => x.signed_duration_since(self.start),
			None => Local::now().signed_duration_since(self.start),
		}
	}

	fn stop(&mut self) {
		match self.end {
			Some(_x) => panic!("Tried to stop block that is already stopped!"),
			None => self.end = Some(Local::now()),
		}
	}

	fn add_tag(&mut self, tag: Tag) {
		self.tags.push(tag);
	}

	fn to_string(&self) -> String {
		let mut msg: String = String::new();
		msg.push_str("Name: ");
		msg += &self.name;
		msg.push_str("\nStart: ");
		msg += &self.start.to_rfc2822();
		msg.push_str("\nStop: ");
		match self.end {
			Some(x) => msg += &x.to_rfc2822(),
			None => msg.push_str("None"),
		}
		msg.push_str("\nTags: ");
		for i in self.tags.iter() {
			msg += &i.name;
			msg.push_str(" ");
		}
		msg.push_str("\nProject: ");
		match self.project.clone() {
			Some(x) => msg += &x.name,
			None => msg.push_str("None"),
		}
		msg.push_str("\n");
		msg
	}
}

// fn confirm(socket: &UdpSocket, src: &SocketAddr) {
// 	socket.send_to(&[1; 1], src);
// }

fn main() {
	let mut cache: Vec<Block> = Vec::new();
	let mut current: Block = Block::new();
	let listener = TcpListener::bind("127.0.0.1:34254").unwrap();
	for stream in listener.incoming() {
		match stream {
			Ok(mut stream) => {
				let mut buf = [0; std::mem::size_of::<Request>()];
				let req: Request;	
				stream.read(&mut buf).unwrap();
				unsafe {req = std::mem::transmute::<[u8; std::mem::size_of::<Request>()], Request>(buf);}
				match req.query {
					Query::ADD(e) => match e {
						Entity::Block(name, proj) => {
							current.stop();
							cache.push(current.clone());
							current = Block::new();
							current.name = unpack(name);
							current.project = Some(Project {name: unpack(proj)});
						},
						Entity::Tag(tag) => current.tags.push(Tag {name: unpack(tag)}),
						Entity::Project(proj) => current.project = Some(Project {name: unpack(proj)}),
					},
					Query::GET(s) => match s {
						Specifier::Relative(rel) => {
							if (rel == 0) {
								stream.write(current.to_string().as_bytes()).unwrap();
							} else {
								stream.write(cache[cache.len() - rel].to_string().as_bytes()).unwrap();
							}
						},
						_ => (),
					},
					_ => (),
				}
            },
            Err(e) => {
                println!("Error: {}", e);
            },
		}
	}
	drop(listener);
}
