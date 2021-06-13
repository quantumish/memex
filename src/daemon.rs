pub mod requests;
pub use crate::requests::*;

use std::thread;
use std::time;
use chrono::{Duration, DateTime, Local};
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{Read, Write};
use nanoid::nanoid;

fn unpack(mut s: Vec<u8>) -> String {
	s.retain(|&x| x != 0);
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
	id: String,
}

impl Block {
	fn new() -> Block {
		let alphabet: [char; 16] = [
			'1', '2', '3', '4', '5', '6', '7', '8', '9', '0', 'a', 'b', 'c', 'd', 'e', 'f'
		];
		Block {
			name: String::new(),
			start: Local::now(),
			end: None,
			tags: Vec::new(),
			project: None,
			id: nanoid!(8, &alphabet),
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

	// fn to_oneline_string(&self) -> String {

	// }

	fn to_detailed_string(&self) -> String {
		let mut msg: String = String::new();
		msg += &format!("[`{}`] *{}*\n", &self.id, &self.name);
		msg += &format!("**Start**: {}\n", &self.start.to_rfc2822());
		msg.push_str("**Stop**: ");
		match self.end {
			Some(x) => {
				msg += &x.to_rfc2822();
				msg.push_str("\n**Duration**: ");
				let dur = x.signed_duration_since(self.start);
				// TODO handle days and weeks
				msg += &format!("{:02}:{:02}:{:02}", dur.num_hours(), dur.num_minutes(), dur.num_seconds());
			}
			None => msg.push_str("None"),
		}
		msg.push_str("\n**Tags**: ");
		for i in self.tags.iter() {
			msg += &i.name;
			msg.push_str(" ");
		}
		msg.push_str("\n**Project**: ");
		match self.project.clone() {
			Some(x) => msg += &x.name,
			None => msg.push_str("None"),
		}
		msg.push_str("\n");
		msg
	}
}

// fn confirm(socket: &UdpSocket, src: &SocketAddr) {
//	socket.send_to(&[1; 1], src);
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
							current.name = unpack(name.to_vec());
							current.project = Some(Project {name: unpack(proj.to_vec())});
						},
						Entity::Tag(tag) => current.tags.push(Tag {name: unpack(tag.to_vec())}),
						Entity::Project(proj) => current.project = Some(Project {name: unpack(proj.to_vec())}),
					},
					Query::GET(s) => match s {
						Specifier::Relative(rel) => {
							if (rel == 0) {
								stream.write(current.to_detailed_string().as_bytes()).unwrap();
							} else {
								stream.write(cache[cache.len() - rel].to_detailed_string().as_bytes()).unwrap();
							}
						},
						Specifier::Id(id) => {
							let ident = unpack(id.to_vec());
							for block in cache.iter() {
								if block.id.eq(&ident) {
									stream.write(block.to_detailed_string().as_bytes()).unwrap();
								}
							}
						}
						_ => (),
					},
					// Query::LOG(r) => match r {
					//	Range::Relative {

					//	}
					// }
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
