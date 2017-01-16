extern crate irc;
extern crate rustc_serialize;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{sleep, spawn};
use std::time::Duration;
use irc::client::prelude::*;

mod phrase;

fn send(server: &IrcServer, limit: &Arc<AtomicBool>, target: &str, msg: &str) {
    if !limit.load(Ordering::Relaxed) {
        server.send_privmsg(target, msg).unwrap();
        println!(":: {}", msg);
        limit.store(true, Ordering::Relaxed);
    }
}

fn main() {
    const PHRASES_FILE: &'static str = "phrases.json";
    // echo phrase setup
    let mut pcon = phrase::PhraseConfig::load(PHRASES_FILE).unwrap();
    // server setup
    let server = IrcServer::new("config.json").unwrap();
    server.identify().unwrap();
    // configuration setup
    let con = server.config().clone();
    let channel = con.channels.unwrap()[0].clone();
    let owner = con.owners.unwrap()[0].clone();
    // start here
    const VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");
    println!("Started Rustyecho (v{}) in channel: {}",VERSION.unwrap_or("unknown"), channel);

    let limit = Arc::new(AtomicBool::new(false));
    let quit = Arc::new(AtomicBool::new(false));
    let limited = limit.clone();
    let q = quit.clone();
    spawn(move || {
        while !q.load(Ordering::Relaxed) {
            sleep(Duration::new(10, 0));
            if limited.load(Ordering::Relaxed) {
                limited.store(false, Ordering::Relaxed);
                println!("ready::");
            }
        }
    });
    for message in server.iter() {
        // Do message processing.
        let message = message.unwrap();
        //print!("{}", message);
        match message.command {
            Command::PRIVMSG(ref target, ref msg) => {
                // if the owner types !quit => exit the program
                let msg_nick = message.source_nickname().unwrap();
                // Check the message if from an owner for a specific response.
                if msg.contains("!") && msg_nick.contains(&owner) {
                    if msg.contains("quit") {
                        server.send_quit("").unwrap();
                        println!("quit:: commanded by {}", msg_nick);
                        quit.store(true, Ordering::Relaxed);
                        return;
                    } else if msg.contains("version") {
                        server.send_privmsg(target, &format!("Echo (Rusty) Reporting for duty (v{})", VERSION.unwrap_or("unknown"))).unwrap();
                    } else if msg.contains("reload") {
                        pcon = phrase::PhraseConfig::load(PHRASES_FILE).unwrap();
                        server.send_privmsg(target, "Reloaded echo list").unwrap();
                    } else if msg.contains("add") {
                        let new_phrase = msg.split("add ").nth(1).unwrap_or("");
                        if new_phrase != "" {
                            println!("add:: {}", new_phrase);
                            let mut new_list = pcon.phrases.clone().unwrap();
                            new_list.push(String::from(new_phrase));
                            let new_pcon = phrase::PhraseConfig {
                                phrases: Some(new_list)
                            };
                            let _ = new_pcon.save(PHRASES_FILE).unwrap();
                            pcon = phrase::PhraseConfig::load(PHRASES_FILE).unwrap();
                            server.send_privmsg(target, &format!("Added {}", new_phrase)).unwrap();
                        }
                    }
                }
                //println!("{}: {}", message.source_nickname().unwrap(), msg);
                // Filter through the phrases to check if they need to be echoed
                let (ret, phrase) = pcon.check(msg);
                if ret {
                    send(&server,&limit,target,phrase.unwrap());
                }
            },
            Command::NOTICE(_, ref msg) => {
                // output server notices
                print!("notice:: {}", msg);
            },
            _ => ()
        }
    }
}
