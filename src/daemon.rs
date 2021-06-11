use std::thread;
use std::time;
use chrono::Duration;
use chrono::DateTime;
use chrono::Local;

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
		tags.push(tag);
	}
}

fn main() {
	println!("sz: {}!", std::mem::size_of::<i32>());
	let mut block = Block::new();
	thread::sleep(time::Duration::from_secs(1));
	println!("{}", block.get_duration());
	block.stop();
}
