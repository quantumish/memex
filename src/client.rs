use std::net::UdpSocket;
pub mod requests;
pub use crate::requests::*;
use std::thread;
use clap::{AppSettings, Clap};
use crossterm::style::{Attribute::*, Color::*};
use termimad::*;
use std::result;

fn pack(s: &String) -> [u8; MAX_NAME] {
	let chars: Vec<char> = s.chars().collect();
	let length = chars.len();
	if (length > MAX_NAME) {panic!("String too big to pack!");}
	let mut tmp: [u8; MAX_NAME] = [0; MAX_NAME];
	for i in 0..length {
		tmp[i] = chars[i] as u8;
	}
	tmp
}

fn fail(skin: &termimad::MadSkin, activity: &str, what: &str) {
	skin.print_inline(&format!("** ✗ Failed to {} ({})!**\n", activity, what)[..]);
	std::process::exit(1);
}

fn success(skin: &termimad::MadSkin, activity: &str) {
	skin.print_inline(&format!("* ✓ Successfully {}!*\n", activity)[..]);
	std::process::exit(0);
}

fn send_request(socket: &UdpSocket, req: Request) -> result::Result<(), &'static str> {
	unsafe {
		let buf = std::mem::transmute::<Request, [u8; std::mem::size_of::<Request>()]>(req);
		socket.send_to(&buf, "127.0.0.1:34254");
	}
	let mut buf: [u8; 16] = [0; 16];
	socket.set_read_timeout(Some(std::time::Duration::new(1,0)));
	match socket.recv_from(&mut buf) {
		Ok(_) => {
			if (buf[0] != 1) {
				return Err("failed to recieve response from daemon");
			}
			Ok(())
		},
		Err(x) => Err("failed to read from socket"),
	}
}

fn add_block(socket: &UdpSocket, name: String, tags: Vec<String>, proj: String) -> result::Result<(), &'static str> {
	let req : Request = Request {
		query: Query::ADD(Entity::Block(pack(&name), pack(&proj))),
	};
	send_request(&socket, req)?;
	for tag in tags.iter() {
		let req : Request = Request {
			query: Query::ADD(Entity::Tag(pack(&tag))),
		};
		send_request(&socket, req)?;
	}
	return Ok(());
}

fn add_tag(socket: &UdpSocket, name: String) -> result::Result<(), &'static str> {
	let req : Request = Request {
		query: Query::ADD(Entity::Tag(pack(&name))),
	};
	send_request(&socket, req)?;
	return Ok(());
}

fn get_block(socket: &UdpSocket) {
	let req : Request = Request {
		query: Query::GET(Specifier::Relative(0)),
	};
	let mut response: [u8; 1024] = [0; 1024];
	let result: (usize, std::net::SocketAddr);
	send_request(&socket, req);
	match socket.recv_from(&mut response) {
		Ok(x) => result = x,
		Err(_) => panic!("AAAAAA"),
	}
	match String::from_utf8(response[..result.0].to_vec()) {
		Ok(x) => print!("aa {}", x),
		Err(_) => panic!("AAhvgdsajssbvc"),
	}
}

#[derive(Clap)]
#[clap(version = "1.0", author = "quantumish")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
	#[clap(subcommand)]
	subcmd: QueryCmd,
}

#[derive(Clap)]
enum QueryCmd {
	Add(Add),
	Get(Get),
}

#[derive(Clap)]
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
struct Block {
	#[clap(short)]
	name: String,
	#[clap(short)]
	tags: String,
	#[clap(short)]
	project: String,
}

#[derive(Clap)]
struct Tag {
	#[clap(short)]
	name: String,
}

#[derive(Clap)]
struct Get {
	#[clap(short, default_value = "0")]
	relative: usize,
}

fn main() {
	let opts: Opts = Opts::parse();
	let mut skin = MadSkin::default();
	skin.italic.set_fg(Green);
	skin.bold.set_fg(Red);
	let socket;
	match UdpSocket::bind("127.0.0.1:34256") {
		Ok(x) => socket = x,
		Err(_) => panic!("AA"),
	}
	match opts.subcmd {
		QueryCmd::Add(query) => {
			match query.subcmd {
				EntityCmd::Block(b) => {
					add_block(&socket, b.name, b.tags.split(",")
							  .map(str::to_string).collect(), b.project);
					success(&skin, "started new block");
				},
				EntityCmd::Tag(t) => {
					match add_tag(&socket, t.name) {
						Ok(_) => success(&skin, "added tag to existing block"),
						Err(s) => fail(&skin, "add tag to existing block", s),
					}
				},
			}
		},
		QueryCmd::Get(_) => get_block(&socket),
	}

}
