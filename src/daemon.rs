pub mod requests;
pub use crate::requests::*;

use std::thread;
use std::time;
use chrono::{Duration, DateTime, Local, Date, NaiveTime};
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

	fn to_oneline_string(&self) -> String {
		return format!("\\* [`{}`] {}-{} *{}*\n",
					   &self.id,
					   &self.start.time().to_string().split(".").collect::<Vec<&str>>()[0], // HACK
					   &self.end.unwrap().time().to_string().split(".").collect::<Vec<&str>>()[0], // HACK
					   &self.name);
	}

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


struct Handler {
	cache: Vec<Block>,
    current: Block,
	listener: TcpListener,
	stream: TcpStream,
}
	
// fn confirm(socket: &UdpSocket, src: &SocketAddr) {
//	socket.send_to(&[1; 1], src);
// }

impl Handler {
	fn new() -> Handler {
		Handler {
			cache: Vec::new();
			current: Block::new();
			listener: TcpListener::bind("127.0.0.1:34254").unwrap();
		}
	}
	
	fn handle_add(e: Entity) {
		match e {
			Entity::Block(name, proj) => {
				current.stop();
				cache.push(current.clone());
				current = Block::new();
				current.name = unpack(name.to_vec());
				current.project = Some(Project {name: unpack(proj.to_vec())});
			},
			Entity::Tag(tag) => current.tags.push(Tag {name: unpack(tag.to_vec())}),
			Entity::Project(proj) => current.project = Some(Project {name: unpack(proj.to_vec())}),
		}
	}

	fn handle_get(s: Specifier) {
		match s {
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
		}
	}

	fn handle_log(r: Range, f: Fmt) {
		match r {
			Range::Term(t) => match t {
				Term::Today => todo!(),
				Term::Yesterday => todo!(),
				Term::Week => todo!(),
				Term::Month => todo!(),
				Term::Year => todo!(),
				Term::All => {
					let mut s: DateTime<Local> = Local::now();
					cache[0] =
						Block {
							name: String::from("Yesterday time"),
							id: String::from("deadbeef"),
							tags: Vec::new(),
							project: None,
							start: Local::now(),
							end: Some(Local::now()),
						};
					cache[0].start = cache[0].start - Duration::days(2) + Duration::hours(1);
					cache[0].end = Some(cache[0].end.unwrap() - Duration::days(2) + Duration::hours(2));
					let mut msg: String = String::new();
					let mut date: DateTime<Local> = cache[cache.len()-1].start + Duration::days(1);
					for i in cache[..].iter().rev() {									
						if (i.start.signed_duration_since(date).num_hours() <= -24) {
							date = i.start;
							msg+=&format!("\n{}\n", i.start.date().format("%B %d, %Y"));
						}
						msg+=&i.to_oneline_string();
					}
					stream.write(msg.as_bytes()).unwrap();
				},
			},
			Range::TimeRange(_, _) => todo!(),
			Range::RelativeRange(beg, end) => todo!(),
		},
	}
}

fn main() {
	let mut handler = Handler::new();
	for stream in listener.incoming() {		
		match stream {
			Ok(mut stream) => {
				handler.stream = stream;
				let mut buf = [0; std::mem::size_of::<Request>()];
				let req: Request;
				stream.read(&mut buf).unwrap();
				unsafe {req = std::mem::transmute::<[u8; std::mem::size_of::<Request>()], Request>(buf);}
				match req.query {
					Query::ADD(e) => handler.handle_add();
					Query::GET(s) => handler.handle_get();
					Query::LOG(r,f) => handler.handle_log();
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
