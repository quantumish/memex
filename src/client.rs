pub mod requests;
pub use crate::requests::*;

pub mod cliargs;
pub use crate::cliargs::*;

use crossterm::style::Color::*;
use termimad::*;
use std::net::TcpStream;
use std::io::{Read, Write};
use anyhow::{Result, Error};
use clap::Clap;

fn send_request(mut stream: &TcpStream, req: Request) -> Result<(), Error> {
    stream.write_all(serde_json::to_string(&req)?.as_bytes())?;
    Ok(())
}

fn get_block(mut stream: &TcpStream, spec: Specifier) -> Result<String> {
    let req : Request = Request {
	query: Query::GET(spec),
    };
    let mut response: [u8; 1024] = [0; 1024];
    send_request(stream, req)?;
    stream.read(&mut response)?;
    Ok(String::from_utf8(response.to_vec())?)
}

fn handle_add(stream: &TcpStream, entity: EntityCmd) -> Result<()>{
    match entity {
	EntityCmd::Block(b) => {
	    send_request(stream, Request {
		query: Query::ADD(Entity::Block(b.name, b.project))
	    })?;
	    for tag in b.tags.split(',').map(str::to_string) {
		send_request(stream, Request {
		    query: Query::ADD(Entity::Tag(tag)),
		})?;
	    }
	}
	_ => ()
    }
    Ok(())
}

fn handle_log(mut stream: &TcpStream) -> Result<String> {
    send_request(&stream, Request {
	query: Query::LOG(Range::Term(Term::All)),
    })?;
    let mut log: String = String::new();
    let mut response: [u8; 64] = [0; 64];
    stream.read_exact(&mut response)?;
    let len = String::from_utf8(response.to_vec())?.parse::<usize>()?;
    let mut read_size: usize = 0;
    let mut response: [u8; 4096] = [0; 4096];
    loop {
	let sz = stream.read(&mut response)?;
	if sz > len-read_size {
	    log += &String::from_utf8(response[..len-read_size].to_vec())?;
	    break;
	} else {
	    log += &String::from_utf8(response.to_vec())?;
	}
	read_size += sz;
    }
    log.push_str("\n");
    Ok(log)
}

fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    let mut skin = MadSkin::default();
    skin.bold.set_fg(Blue);
    skin.italic.set_fg(Blue);
    skin.inline_code.set_fg(Cyan);
    skin.inline_code.set_bg(Black);
    let stream = TcpStream::connect(opts.ip.clone()+":5000")?;
    match opts.subcmd {
	QueryCmd::Add(query) => handle_add(&stream, query.subcmd)?,
	QueryCmd::Get(g) => {
	    let mut spec = Specifier::Relative(0);
	    if let Some(r) = g.rel {
		spec = Specifier::Relative(r);
	    } else if let Some(id) = g.id {
		spec = Specifier::Id(id);
	    }
	    skin.print_inline(&get_block(&stream, spec)?);
	},
	QueryCmd::Log(_r) => skin.print_inline(&handle_log(&stream)?[2..]), 
    }
    Ok(())
}
