use rust_http::{common::{HttpError, HttpResult, HttpSocket, /*Stream*/}, websocket::WebSocketFrameType};

use crate::structs::SharedData;
// use std::{collections::HashMap};


pub fn available(path: &str)->Option<&'static str>{
    match path{
        s if s.starts_with("/internal/example")=>Some("example"),
        s if s.starts_with("/websocket/echo")=>Some("ws-echo"),
        _=>None,
    }
}

pub async fn call<S:HttpSocket+Sized+Send+'static>(name: &str, shared: &SharedData, path: &str, mut res: S)->HttpResult<()>{
    match name{
        "example"=>example(shared, path, res).await,
        "ws-echo"=>ws_echo(shared, path, res).await,
        _=>{
            res.set_status(500, "Internal server error".to_owned())?;
            res.close(b"Internal server error\n\nendpoint does not exist\n").await?;
            Err(HttpError::Invalid)
        },
    }
}


async fn example<S:HttpSocket>(_shared: &SharedData, _path: &str, mut res: S)->HttpResult<()>{
    res.close(b"example endpoint\n").await
}

async fn ws_echo<S:HttpSocket+Sized+Send+'static>(_shared: &SharedData, _path: &str, mut res: S)->HttpResult<()>{
    let c=res.get_client().await?;
    match c.headers.get("upgrade").map(|h|h[0].as_str()).as_deref(){
        Some("websocket")=>{
            let mut ws=res.websocket().await?;
            println!("ws-echo: started websocket");
            loop{
                let frames=ws.incoming().await?;
                if frames.is_empty(){ break }
                for frame in frames{
                    println!("ws-echo: received ws frame {:?} {}",frame.ftype,frame.payload.len());
                    match frame.ftype{
                        WebSocketFrameType::Ping=>ws.send_pong(frame.get_payload()).await?,
                        WebSocketFrameType::Text=>ws.send_text(frame.get_payload()).await?,
                        WebSocketFrameType::Binary=>ws.send_text(frame.get_payload()).await?,
                        _=>()
                    }
                }
            }
            // let mut ws=res.websocket().await?;
            // loop{
            //     let frames=ws.incoming().await?;
            //     if frames.is_empty(){ break }
            //     for frame in frames{
            //         match frame.ftype{
            //             WebSocketFrameType::Ping=>ws.send_pong(frame.get_payload()).await?,
            //             WebSocketFrameType::Text=>ws.send_text(frame.get_payload()).await?,
            //             WebSocketFrameType::Binary=>ws.send_text(frame.get_payload()).await?,
            //             _=>()
            //         }
            //     }
            // }
            Ok(())
        },
        _=>res.close(b"websocket").await,
    }
    // Ok(())
}


// pub type Middleware<S> = fn(&SharedData, &str, S) -> std::pin::Pin<Box<dyn std::future::Future<Output = HttpResult<()>> + Send>>;

// pub struct MiddlewareMap<S:HttpSocket> {
//     funcs: HashMap<String, Middleware<S>>,
// }

// impl<S:HttpSocket> MiddlewareMap<S> {
//     pub fn new() -> Self {
//         Self {
//             funcs: HashMap::new(),
//         }
//     }

//     pub fn register(&mut self, name: &str, f: Middleware<S>) {
//         self.funcs.insert(name.to_string(), f);
//     }

//     pub async fn call(&self, name: &str, shared: &SharedData, path: &str, mut res: S) -> HttpResult<()> {
//         if let Some(func) = self.funcs.get(name) {
//             func(shared, path, res).await
//         } else {
//             res.set_status(500, "Internal server error".to_owned())?;
//             res.close(b"Internal server error\n\nendpoint does not exist\n").await?;
//             Err(HttpError::Invalid)
//         }
//     }
// }
