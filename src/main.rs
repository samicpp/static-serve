mod mime_map;
mod handlers;
mod structs;

use rust_http::{
    http1::handler::Http1Socket, listener, /*traits::HttpSocket*/
};

use std::{
    env,
    path::Path,
    sync::Arc, time::{Instant},
};

use crate::{mime_map::mime_map, structs::SharedData};

#[tokio::main]
async fn main()->std::io::Result<()> {
    let start=Instant::now();

    let _ = dotenvy::from_path(Path::new(".env"));
    let mut serve_dir: String = env::var("SERVE_DIR").unwrap_or("./public".to_string());
    let mut address = env::var("ADDRESS").unwrap_or("0.0.0.0:1024".to_string());
    let args: Vec<String> = env::args().collect();

    if args.len()==2 && (args[1]=="-h"||args[1]=="--help"){
        println!("\t");
        println!("\x1b[32musage\x1b[0m: {} address directory",args[0]);
        println!("\x1b[33mexample\x1b[0m: {} 0.0.0.0:2000 ./files",args[0]);
        println!("\x1b[34mdefault\x1b[0m: {} 0.0.0.0:1024 ./public",args[0]);
        println!("\x1b[35mparameters can also be passed down through environmental variable ADDRESS and SERVE_DIR\x1b[0m");
        println!("\x1b[36m.env files for vars supported\x1b[0m");
        println!("\t");
        std::process::exit(0);
        // return Ok(())
    }
    if args.len() > 1 {
        let host_str = args[1].clone();
        address=host_str;
    } if args.len() > 2 {
        serve_dir = args[2].clone();
    }


    println!(
        "Parameters of the server are\n\x1b[32maddress = {:?}\n\x1b[34mdirectory = {}\x1b[0m",
        address, serve_dir
    );

    let shared=Arc::new(SharedData{
        mime: mime_map(), 
        serve_dir,
    });

    let listener = {
        let shared=Arc::clone(&shared);
        move |conn: Http1Socket| {
            let shared=Arc::clone(&shared);
            async move {
                let now=Instant::now();
                let res = handlers::handler(&shared, conn).await;
                println!("\x1b[36mhandler took {}ms\x1b[0m",now.elapsed().as_nanos() as f64 /1000000.0);
                match res {
                    Ok(())=>println!("\x1b[32mhandler didnt error\x1b[0m"),
                    Err(err)=>eprintln!("\x1b[31mhandler errored\n{:?}\x1b[0m",err),
                };
            }
        }
    };

    ctrlc::set_handler(move||{
        println!("\x1b[31mSIG_INT received\x1b[0m\n\x1b[36mprocess exit after {}s\x1b[0m",&start.elapsed().as_millis()/1000);
        std::process::exit(0);
    }).expect("couldnt set ctrl+c handler");

    println!("http://{}/",&address);
    listener::http_listener(&address, listener).await.unwrap();
    
    

    // println!("process exit after {}s",&start.elapsed().as_millis()/1000);

    Ok(())
}
