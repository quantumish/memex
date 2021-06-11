use std::net::UdpSocket;
pub mod requests;
pub use crate::requests::*;
use std::thread;
use clap::{AppSettings, Clap};


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

fn send_request(socket: &UdpSocket, req: Request) {
	unsafe {
		let buf = std::mem::transmute::<Request, [u8; std::mem::size_of::<Request>()]>(req);
		socket.send_to(&buf, "127.0.0.1:34254");
	}
}

fn add_block(socket: &UdpSocket, name: String, tags: Vec<String>, proj: String) {
	let req : Request = Request {
		query: Query::ADD,
		entity: Entity::Block(pack(&name), pack(&proj)),
	};
	send_request(&socket, req);
	for tag in tags.iter() {
		let req : Request = Request {
			query: Query::ADD,
			entity: Entity::Tag(pack(&tag)),
		};
		send_request(&socket, req);
	}
}

fn get_block(socket: &UdpSocket) {
	let req : Request = Request {
		query: Query::GET,
		entity: Entity::Block(pack(&String::new()), pack(&String::new()))
	};
	let mut response: [u8; 1024] = [0; 1024];
	send_request(&socket, req);
	let result: (usize, std::net::SocketAddr);
	match socket.recv_from(&mut response) {
		Ok(x) => result = x,
		Err(_) => panic!("AAAAAA"),
	}
	match String::from_utf8(response[..result.0].to_vec()) {
		Ok(x) => print!("{}", x),
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
struct Get {}

fn main() {
	let opts: Opts = Opts::parse();
	let socket;
	match UdpSocket::bind("127.0.0.1:34256") {
		Ok(x) => socket = x,
		Err(_) => panic!("AA"),
	}
	match opts.subcmd {
		QueryCmd::Add(e) => {
			match e {
				Block => add_block(&socket, b.name, b.tags.split(",")
										   .map(str::to_string).collect(), b.project),
				QueryCmd::Get => get_block(&socket),
			}
		}
	}

}
