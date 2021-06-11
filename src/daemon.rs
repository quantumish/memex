use std::time;
use std::thread;

struct Tag {
	name: String
}

struct Block {
	start: time::SystemTime,
	end: Option<time::SystemTime>,
	tags: Vec<Tag>,
}

impl Block {
	fn new() -> Block {
		Block {
			start: time::SystemTime::now(),
			end: None
		}
	}

	fn get_duration(&self) -> time::Duration {
		match self.end {
			Some(x) => {
				match self.end.duration_since(self.start) {
					Ok(duration) => duration,
					Err(_e) => panic!("Subtraction somehow doesn't work.")
				}
			}
			None => {
				match self.start.elapsed() {
					Ok(duration) => duration,
					Err(_e) => panic!("Handling errors is for wimps.")
				}
			}
		}
	}

	fn stop(&self) {
		self.end = time::SystemTime::now();
	}
}

fn main() {
	println!("Hello, world!");
	let block = Block::new();
	thread::sleep(time::Duration::from_secs(1));
	println!("{}", block.get_duration().as_secs());
	block.stop();
}
