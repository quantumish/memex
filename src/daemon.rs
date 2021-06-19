pub mod requests;
pub use crate::requests::*;

use chrono::{Duration, DateTime, Local};
use std::mem;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write, BufRead, Seek};
use nanoid::nanoid;
use flexi_logger::*;
use log::*;
use std::fs::*;
use::std::path::Path;

const MAX_CACHE_LEN: u32 = 1;

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

	fn from(s: &String) -> Block {
		return Block::new(); // TODO
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

	fn to_oneline_string(&self) -> String {
		return format!("\\* [`{}`] {}-{} *{}*\n",
					   &self.id,
					   &self.start.time().to_string().split(".").collect::<Vec<&str>>()[0], // HACK
					   &self.end.unwrap_or(Local::now()).time().to_string().split(".").collect::<Vec<&str>>()[0], // HACK
					   &self.name);
	}

	fn to_detailed_string(&self) -> String {
		let mut msg: String = String::new();
		msg += &format!("[`{}`] *{}*\n", &self.id, &self.name);
		msg += &format!("**Start**: {}\n", &self.start.to_rfc2822());
		msg.push_str("**Stop**: ");
		match self.end {
			Some(x) => msg += &x.to_rfc2822(),
			None => msg.push_str("None"),
		}
		let dur: Duration = self.get_duration();
		msg += &format!("\n**Duration**: {:02}:{:02}:{:02}", dur.num_hours(), dur.num_minutes(), dur.num_seconds());
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

fn write_stream(mut stream: &TcpStream, msg: String)
{
	match stream.write(msg.as_bytes()) {
		Err(s) => error!("Failed to write to stream: {}", s),
		_ => (),
	}
}

// TIMESTAMP|TIMESTAMP|"name purposely | causing abuse"|Memex|test,rust

struct Handler {
	cache: Vec<Block>,
	file: File,
	current: Option<Block>
}

impl Handler {
	fn new() -> Handler {
		Handler {
			cache: Vec::new(),
			file: File::open(&Path::new("data.txt")).unwrap(), // TODO Handle errors.
			current: None,
		}
	}

	fn get(&self, rel: usize) -> std::result::Result<Block, &'static str> {
		if rel == 0 {
			if let Some(c) = self.current.clone() {
				return Ok(c);
			} else {return Err("No current block");}
		} else if rel <= self.cache.len() {
			return Ok(self.cache[self.cache.len()-rel-1].clone());
		} else {
			let mut reader = std::io::BufReader::new(&self.file);
			for _i in self.cache.len()+1..rel {
				let mut line = String::new();
				if let Ok(_sz) = reader.read_line(&mut line) {
					return Ok(Block::from(&line));
				} else {return Err("No block found.")};
			}
		}
		Err("No block found.")
	}

	fn add_new(&mut self, b: Block) -> std::result::Result<(), &'static str> {
		if let Some(cur) = self.current.clone() {
			self.cache.push(cur);
			if self.cache.len() > MAX_CACHE_LEN as usize {
				let mut writer = std::io::BufWriter::new(&self.file);
				writer.seek(std::io::SeekFrom::End(0));
				writer.write(self.cache[0].to_oneline_string().as_bytes());
			}
		}
		Ok(())
	}

	fn handle_add(&mut self, stream: &TcpStream, e: Entity) {
		match e {
			Entity::Block(name, proj) => {
				if let Some(i) = &mut self.current {
					i.stop();
					self.cache.push(i.clone());
				}
				let mut tmp: Block = Block::new();
				tmp.name = unpack(name.to_vec());
				tmp.project = Some(Project {name: unpack(proj.to_vec())});
				self.current = Some(tmp);
			},
			Entity::Tag(tag) => {
				if let Some(i) = &mut self.current {
					i.tags.push(Tag {name: unpack(tag.to_vec())})
				} else {write_stream(stream, String::from("ERR: no existing block"));}
			}
			Entity::Project(proj) => {
				if let Some(i) = &mut self.current {
					i.project = Some(Project {name: unpack(proj.to_vec())});
				} else {write_stream(stream, String::from("ERR: no existing block"));}
			}
		}
	}

	fn handle_get(&self, stream: &TcpStream, s: Specifier) {
		match s {
			Specifier::Relative(rel) => {
				if rel == 0 {
					if let Some(i) = &self.current {
						write_stream(stream, i.to_detailed_string());
					} else {write_stream(stream, String::from("ERR: no existing block"));}
				} else {
					if rel > self.cache.len() {
						write_stream(stream, String::from("ERR: out of range"));
					} else {
						write_stream(stream, self.cache[self.cache.len()-rel-1].to_detailed_string());
					}
				}
			},
			Specifier::Id(id) => {
				let ident = unpack(id.to_vec());
				for block in self.cache.iter() { // TODO hashmap
					if block.id.eq(&ident) {
						write_stream(stream, block.to_detailed_string());
					}
				}
				write_stream(stream, String::from("ERR: invalid ID"));
			}
		}
	}

	fn handle_log(&self, stream: &TcpStream, r: Range, _f: Fmt) { // TODO use fmt
		match r {
			Range::Term(t) => match t {
				Term::All => {
					let mut msg: String = String::new();
					let mut date: DateTime<Local> = self.cache[self.cache.len()-1].start + Duration::days(1);
					for i in self.cache[1..].iter().rev() {
						if i.start.signed_duration_since(date).num_hours() <= -24 {
							date = i.start;
							if i.id.ne(&self.cache[self.cache.len()-1].id) {
								msg.push_str("\n");
							}
							msg+=&format!("{}\n", i.start.date().format("%B %d, %Y"));
						}
						msg+=&i.to_oneline_string();
					}
					write_stream(stream, msg);
				},
				_ => (),
			},
			_ => (),
		}
	}
}

// impl Iterator for Handler {
//	type Item = Block;
//	fn next(&mut self) -> Option<Block> {

//	}
// }

fn main() {
	Logger::try_with_env_or_str("info").unwrap()
		.log_to_file(FileSpec::default().basename("memexd").directory("logs"))
		.start().unwrap();
	let mut handler = Handler::new();
	let listener = TcpListener::bind("127.0.0.1:34254").unwrap();
	for stream in listener.incoming() {
		match stream {
			Ok(mut stream) => {
				let mut buf = [0; mem::size_of::<Request>()];
				let req: Request;
				if let Err(s) = stream.read(&mut buf) {
					error!("Faile to read from stream: {}", s);
				}
				unsafe {req = mem::transmute::<[u8; mem::size_of::<Request>()], Request>(buf);}
				match req.query {
					Query::ADD(e) => handler.handle_add(&stream, e),
					Query::GET(s) => handler.handle_get(&stream, s),
					Query::LOG(r,f) => handler.handle_log(&stream, r, f),
				}
			},
			Err(e) => error!("Error in opening stream: {}", e),
		}
	}
	drop(listener);
}
