use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
struct Entry {
	description: String,
	tags: Vec<String>,
	pid: u64,
	created_with: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Post {
	time_entry: Entry,
}

pub async fn get_projects(token: String, workspace: u64) -> HashMap<String, u64> {
	let mut map: HashMap<String, u64> = HashMap::new();
	let client = reqwest::Client::new();
	let res = client.get(String::from("https://api.track.toggl.com/api/v9/workspaces/")+&workspace.to_string()+"/projects")
		.basic_auth(token, Some("api_token"))
		.send().await.unwrap().text().await.unwrap();
	let json: Vec<serde_json::Value> = serde_json::from_str(&res).unwrap();
	for i in json {
		map.insert(i["name"].as_str().unwrap().to_string(),
				   i["id"].as_u64().unwrap()); // HACK
	}
	return map;
}

pub async fn get_proj_id(token: String, map: HashMap<String, u64>, name: String) -> u64 {
	if let Some(id) = map.get(&name) {
		return *id;
	}
	let client = reqwest::Client::new();	
	let res: serde_json::Value = client.post("https://api.track.toggl.com/api/v8/projects")
		.header("Content-Type", "application/json")
		.basic_auth(token, Some("api_token"))
		.body(String::from("{\"project\":{\"name\":\"")+&name+"\", \"wid\": 3816613}}")
		.send().await.unwrap()
		.json().await.unwrap();
	return res["data"]["id"].as_u64().unwrap();
} 

pub async fn get_toggl(token: String) {
	let client = reqwest::Client::new();
	let res = client.get("https://api.track.toggl.com/api/v8/time_entries/current")		
		.basic_auth(token, Some("api_token"))
		.send().await.unwrap().text().await;
}

pub async fn set_toggl(token: String, name: String, pid: u64) {
	let client = reqwest::Client::new();
	let post = Post {
		time_entry: Entry {
			description: name,
			tags: Vec::new(),
			pid: pid,
			created_with: String::from("memex"),
		}
	};
	let res = client.post("https://api.track.toggl.com/api/v8/time_entries/start")
		.header("Content-Type", "application/json")
		.basic_auth(token, Some("api_token"))
		.json(&post)
		.send().await;
}

pub async fn update_toggl(token: String, name: String, tags: Vec<String>, pid: u64) {
	let client = reqwest::Client::new();
	let res = client.get("https://api.track.toggl.com/api/v8/time_entries/current")		
		.basic_auth(token.clone(), Some("api_token"))
		.send().await.unwrap().text().await.unwrap();
	let cur: serde_json::Value = serde_json::from_str(&res).unwrap();
	let client = reqwest::Client::new();
	let post = Post {
		time_entry: Entry {
			description: name,
			tags: tags,
			pid: pid,
			created_with: String::from("memex"),
		}
	};
 	let res = client.put(String::from("https://api.track.toggl.com/api/v8/time_entries/")+&cur["data"]["id"].to_string())
		.header("Content-Type", "application/json")
		.basic_auth(token.clone(), Some("api_token"))
		.json(&post)
		.send().await;
}

