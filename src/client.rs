pub mod requests;
pub use crate::requests::*;
use clap::{AppSettings, Clap};
use crossterm::style::Color::*;
use termimad::*;
use std::net::TcpStream;
use std::io::{Read, Write};

fn pack_attr(s: &String) -> std::result::Result<[u8; MAX_ATTR_NAME], &'static str> {
	let chars: Vec<char> = s.chars().collect();
	let length = chars.len();
	if length > MAX_ATTR_NAME {return Err("String too big to pack!");}
	let mut tmp: [u8; MAX_ATTR_NAME] = [0; MAX_ATTR_NAME];
	for i in 0..length {
		tmp[i] = chars[i] as u8;
	}
	Ok(tmp)
}

fn pack_name(s: &String) -> std::result::Result<[u8; MAX_NAME], &'static str> {
	let chars: Vec<char> = s.chars().collect();
	let length = chars.len();
	if length > MAX_NAME {return Err("String too big to pack!");}
	let mut tmp: [u8; MAX_NAME] = [0; MAX_NAME];
	for i in 0..length {
		tmp[i] = chars[i] as u8;
	}
	Ok(tmp)
}


fn fail(skin: &termimad::MadSkin, activity: &str, what: &str) {
	skin.print_inline(&format!("** ✗ Failed to {} ({})!**\n", activity, what)[..]);
	std::process::exit(1);
}

fn success(skin: &termimad::MadSkin, activity: &str) {
	skin.print_inline(&format!("* ✓ Successfully {}!*\n", activity)[..]);
	std::process::exit(0);
}

fn send_request(mut stream: &TcpStream, req: Request) -> std::result::Result<(), &'static str> {
	unsafe {
		let buf = std::mem::transmute::<Request, [u8; std::mem::size_of::<Request>()]>(req);
		stream.write(&buf).unwrap();
	}
	// let mut buf: [u8; 16] = [0; 16];
	// stream.set_read_timeout(Some(std::time::Duration::new(1,0)));
	// match stream.read(&mut buf) {
	//	Ok(_) => {
	//		if (buf[0] != 1) {
	//			return Err("failed to recieve response from daemon");
	//		}
	//		Ok(())
	//	},
	//	Err(x) => Err("failed to read from socket"),
	// }
	Ok(())
}

fn add_block(stream: &TcpStream, name: String, proj: String) -> std::result::Result<(), &'static str> {
	let req : Request = Request {
		query: Query::ADD(Entity::Block(pack_name(&name).unwrap(), pack_attr(&proj).unwrap())),
	};
	send_request(&stream, req)?;
	Ok(())
}

fn add_tag(stream: &TcpStream, name: String) -> std::result::Result<(), &'static str> {
	let req : Request = Request {
		query: Query::ADD(Entity::Tag(pack_attr(&name).unwrap())),
	};
	send_request(&stream, req)?;
	Ok(())
}

fn get_block(mut stream: &TcpStream, spec: Specifier) -> std::result::Result<(), String> {
	let req : Request = Request {
		query: Query::GET(spec),
	};
	let mut response: [u8; 1024] = [0; 1024];
	if let Err(s) = send_request(&stream, req) {return Err(s.to_string());}
	if let Err(s) = stream.read(&mut response) {return Err(s.to_string());}
	let stringified = String::from_utf8(response.to_vec()).unwrap();
	if stringified[..3].eq("ERR") {
		return Err(stringified[5..].to_string());
	}
	let mut skin = MadSkin::default();
	skin.bold.set_fg(Blue);
	skin.italic.set_fg(Blue);
	skin.inline_code.set_fg(Cyan);
	skin.inline_code.set_bg(Black);
	skin.print_inline(&stringified);
	Ok(())
}

#[derive(Clap)]
#[clap(version = "1.0", author = "quantumish")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
	#[clap(long, default_value="localhost")]
	ip: String,
	#[clap(subcommand)]
	subcmd: QueryCmd,
}

#[derive(Clap)]
enum QueryCmd {
	Add(Add),
	Get(Get),
	Log(Log),
}

#[derive(Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
struct Add {
	#[clap(subcommand)]
	subcmd: EntityCmd,
}

#[derive(Clap)]
enum EntityCmd {
	Block(Block),
	Tag(Tag),
}

#[derive(Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
struct Block {
	#[clap(short)]
	name: String,
	#[clap(short)]
	tags: String,
	#[clap(short)]
	project: String,
}

#[derive(Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
struct Tag {
	#[clap(short)]
	name: String,
}

#[derive(Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
struct Get {
	#[clap(long)]
	rel: Option<usize>,
	#[clap(long)]
	id: Option<String>,
}

#[derive(Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
struct Log {
	// #[clap(long)]
	// compact: bool,
}

fn main() {
	let opts: Opts = Opts::parse();
	let mut skin = MadSkin::default();
	skin.italic.set_fg(Green);
	skin.bold.set_fg(Red);
	let mut stream = TcpStream::connect(opts.ip.clone()+":5000").unwrap();
	match opts.subcmd {
		QueryCmd::Add(query) => {
			match query.subcmd {
				EntityCmd::Block(b) => {
					if let Err(s) = add_block(&stream, b.name, b.project) {fail(&skin, "start new block", s);}
					for tag in b.tags.split(",").map(str::to_string) {
						let stream = TcpStream::connect(opts.ip.clone()+":5000").unwrap();
						if let Err(s) = add_tag(&stream, tag) {fail(&skin, "add tag to new block", s);}
					}
					success(&skin, "started new block");
				},
				EntityCmd::Tag(t) => {
					match add_tag(&stream, t.name) {
						Ok(_) => success(&skin, "added tag to existing block"),
						Err(s) => fail(&skin, "add tag to existing block", s),
					}
				},
			}
		},
		QueryCmd::Get(g) => {
			let result: std::result::Result<(), String>;
			if g.rel.is_some() {
				result = get_block(&stream, Specifier::Relative(g.rel.unwrap()));
			} else if g.id.is_some() {
				result = get_block(&stream, Specifier::Id(pack_attr(&g.id.unwrap()).unwrap()));
			} else {
				result = get_block(&stream, Specifier::Relative(0));
			}
			if let Err(s) = result {fail(&skin, "get existing block", &s);}
		},
		QueryCmd::Log(_r) => { // TODO add ranges
			let req : Request = Request {
				query: Query::LOG(Range::Term(Term::All)),
			};
			send_request(&stream, req).unwrap();
			let mut log: String = String::new();
			let mut response: [u8; 64] = [0; 64];
			stream.read(&mut response).unwrap();
			let len = String::from_utf8(response.to_vec()).unwrap().parse::<usize>().unwrap();
			let mut read_size: usize = 0;
			let mut response: [u8; 4096] = [0; 4096];
			loop {
				let sz = stream.read(&mut response).unwrap();
				if sz > len-read_size {
					log += &String::from_utf8(response[..len-read_size].to_vec()).unwrap();
					break;
				} else {
					log += &String::from_utf8(response.to_vec()).unwrap();
				}
				read_size += sz;
			}
			log.push_str("\n");
			let mut skin = MadSkin::default();
			skin.bold.set_fg(Blue);
			skin.italic.set_fg(Blue);
			skin.inline_code.set_fg(Cyan);
			skin.inline_code.set_bg(Black);
			skin.print_inline(&log[2..]); // HACK
		}
	}
}
