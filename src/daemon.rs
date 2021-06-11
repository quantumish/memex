pub mod requests;
pub use crate::requests::*;


use std::thread;
use std::time;
use chrono::{Duration, DateTime, Local};
use std::net::UdpSocket;

struct Tag {
	name: String
}

struct Project {
	name: String
}

pub struct Block {
	start: DateTime<Local>,
	end: Option<DateTime<Local>>,
	tags: Vec<Tag>,
	project: Option<Project>,
}

impl Block {
	fn new() -> Block {
		Block {
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
}

fn main() -> std::io::Result<()> {
	let current: Block;
	let mut socket = UdpSocket::bind("127.0.0.1:34254")?;
	loop {
		// let mut buf = [0; std::mem::size_of::<Request>];
		// let (amt, src) = socket.recv_from(&mut buf)?;
			
	}	
}
