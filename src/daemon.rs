pub mod requests;
pub use crate::requests::*;

use chrono::{Duration, DateTime, Local};
use std::mem;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write, BufRead};
use nanoid::nanoid;
use flexi_logger::*;
use log::*;
use std::fs::{OpenOptions};
use serde::{Serialize, Deserialize};
use config::*;
use std::env;

const MAX_CACHE_LEN: u32 = 1;
const LOG_FMT_STRING: &'static str = "\\* [`%i`] %S-%E *%n* (%p)\n";
const DISPLAY_FMT_STRING: &'static str =
	"[`%i`] *%n*\n**Start**: %s\n**Stop**: %e\n**Tags**: %t\n**Project**: %p\n";

fn unpack(mut s: Vec<u8>) -> String {
	s.retain(|&x| x != 0);
	match std::str::from_utf8(&s) {
		Ok(x) => String::from(x),
		Err(_x) => panic!("Failed to unpack."),
	}
}

#[derive(Serialize, Deserialize, Clone)]
struct Tag {
	name: String
}

impl Tag {
	fn to_string(&self) -> String {
		self.name.clone()
	}
}

#[derive(Serialize, Deserialize, Clone)]
struct Project {
	name: String
}

impl Project {
	fn to_string(&self) -> String {
		self.name.clone()
	}
}

#[derive(Serialize, Deserialize, Clone)]
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

	fn to_format(&self, fmt: String) -> String {
		let end: String;
		match &self.end {
			Some(e) => end = e.to_rfc2822(),
			None => end = String::from("None"),
		};
		return fmt
			.replace("%i", &self.id)
			.replace("%s", &self.start.to_rfc2822())
			.replace("%e", &end)
			.replace("%S", &self.start.time().to_string().split(".").collect::<Vec<&str>>()[0]) // HACK
			.replace("%E", &self.end.unwrap_or(Local::now()).time().to_string().split(".").collect::<Vec<&str>>()[0]) // HACK
			.replace("%N", &Local::now().to_rfc2822())
			.replace("%n", &self.name)
			.replace("%t", &self.tags.clone().into_iter().map(|t| t.to_string())
					 .collect::<Vec<String>>().join(" ").clone())
			.replace("%p", &self.project.clone().unwrap().to_string())
	}

	fn stop(&mut self) {
		match self.end {
			Some(_x) => panic!("Tried to stop block that is already stopped!"),
			None => self.end = Some(Local::now()),
		}
	}
}

fn write_stream(mut stream: &TcpStream, msg: String)
{
	match stream.write(msg.as_bytes()) {
		Err(s) => error!("Failed to write to stream: {}", s),
		_ => (),
	}
}

struct Handler {
	settings: Config,
	cache: Vec<Block>,
	file: String,
	current: Option<Block>
}

impl Handler {
	fn new() -> Handler {
		Handler {
			settings: Config::default(),
			cache: Vec::new(),
			file: String::from("data.json"),
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
			let mut reader = std::io::BufReader::new(
				OpenOptions::new().read(true).open(&self.file).unwrap());
			let mut line = String::new();
			for _i in self.cache.len()..rel {
				line.clear();
				if let Err(_) = reader.read_line(&mut line) {
					return Err("No block found.")
				};
			}
			let out: Block = serde_json::from_str(&line.trim()).unwrap();
			return Ok(out.clone());
		}
	}

	fn add_new(&mut self, name: String, proj: String) -> std::result::Result<(), &'static str> {
		if let Some(cur) = &mut self.current {
			cur.stop();
			self.cache.push(cur.clone());
			if self.cache.len() > MAX_CACHE_LEN as usize {
				let mut writer = std::io::BufWriter::new(
					OpenOptions::new().append(true).create(true).
						open(self.file.clone()+&String::from(".tmp")).unwrap());
				let mut reader = std::io::BufReader::new(
					OpenOptions::new().read(true).open(&self.file).unwrap());
				writer.write((serde_json::to_string(&self.cache[0]).unwrap()+"\n").as_bytes()).unwrap();
				for line in reader.lines() {
					writer.write((line.unwrap()+"\n").as_bytes());
				}
				std::fs::rename("data.json.tmp", "data.json");
				self.cache.drain(0..1);
			}
		}
		let mut tmp: Block = Block::new();
		tmp.name = name;
		tmp.project = Some(Project {name: proj});
		self.current = Some(tmp);
		Ok(())
	}

	fn handle_add(&mut self, stream: &TcpStream, e: Entity) {
		match e {
			Entity::Block(name, proj) =>
				self.add_new(unpack(name.to_vec()), unpack(proj.to_vec())).unwrap(),
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
		let format: String = self.settings.get::<String>("get.format").unwrap_or(DISPLAY_FMT_STRING.to_string());
		match s {
			Specifier::Relative(rel) => {
				if rel == 0 {
					if let Some(i) = &self.current {
						write_stream(stream, i.to_format(format.clone()));
					} else {write_stream(stream, String::from("ERR: no existing block"));}
				} else {
					write_stream(stream, self.get(rel).unwrap().to_format(format.clone()));
				}
			},
			Specifier::Id(id) => {
				let ident = unpack(id.to_vec());
				for block in self.iter() {
					if block.id.eq(&ident) {
						write_stream(stream, block.to_format(format.clone()));
					}
				}
				write_stream(stream, String::from("ERR: invalid ID"));
			}
		}
	}

	fn handle_log(&self, mut stream: &TcpStream, r: Range) { // TODO use fmt
		match r {
			Range::Term(t) => match t {
				Term::All => {
					let mut msg: String = String::new();
					let mut date: DateTime<Local> = Local::now() + Duration::days(1);
					for i in self.iter() {
						if i.start.signed_duration_since(date).num_hours() <= -24 {
							date = i.start;
							msg+=&format!("\n{}\n", i.start.date().format("%B %d, %Y"));
						}
						msg+=&i.to_format(self.settings.get::<String>("log.format").unwrap_or(LOG_FMT_STRING.to_string()));
					}
					write_stream(stream, format!("{:0width$}\n", msg.len(), width=64));
					write_stream(stream, msg);
				},
				_ => (),
			},
			_ => (),
		}
	}

	fn iter(&self) -> std::iter::Chain<impl Iterator<Item = Block> + '_, impl Iterator<Item = Block>> {
		let mut reader = std::io::BufReader::new(
			OpenOptions::new().read(true).open(&self.file).unwrap());
		self.cache.iter().cloned().
			chain(reader.lines()
				  .map(|line| serde_json::from_str(&line.unwrap().trim()).unwrap()))
	}
}

fn main() {
	Logger::try_with_env_or_str("info").unwrap()
		.log_to_file(FileSpec::default().basename("memexd").directory("logs"))
		.start().unwrap();
	let mut handler = Handler::new();
	handler.settings.merge(File::with_name(&(env::var("HOME").unwrap()+"/.config/memex.toml"))).unwrap();
	let listener = TcpListener::bind("0.0.0.0:5000").unwrap();
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
					Query::LOG(r) => handler.handle_log(&stream, r),
				}
			},
			Err(e) => error!("Error in opening stream: {}", e),
		}
	}
	drop(listener);
}
