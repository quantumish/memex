
use serde::{Deserialize, Serialize};
// use hyper::header::{Headers, Authorization, Bearer};

#[derive(Debug, Serialize, Deserialize)]
struct Entry {
	description: String,
	tags: Vec<String>,
	project: i32,
	created_with: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Post {
	entry: Entry,
}

#[tokio::main]
async fn main() {
	let client = reqwest::Client::new();
	let post = Post {
		entry: Entry {
			description: String::from("Testing Memex"),
			tags: vec!(String::from("Code"), String::from("Rust"), String::from("Debugging")),
			project: 0,
			created_with: String::from("memex"),
		}
	};
	let res = client.post("https://api.track.toggl.com/api/v8/time_entries/start")
		.json(&post)
		.header("Authorization", "Basic ????")
		.header("Content-Type", "application/json")
		.send().await;
	println!("{:?}", res);
}
