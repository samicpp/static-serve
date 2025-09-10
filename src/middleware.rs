use std::sync::Arc;

use rust_http::{common::{HttpError, HttpResult, HttpSocket, Stream, /*Stream*/}, websocket::{WebSocket, WebSocketFrameType}};
use tokio::sync::Mutex;

use crate::structs::SharedData;
// use std::{collections::HashMap};

pub type SharedClients<S> = Arc<Mutex<Vec<Arc<WebSocket<S>>>>>;

pub struct MiddlewareData<S:Stream>{
    pub clients: SharedClients<S>,
}
impl<S:Stream> MiddlewareData<S>{
    pub fn empty()->Self{
        Self{
            clients: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

pub fn available(path: &str)->Option<&'static str>{
    match path{
        s if s.starts_with("/internal/example")=>Some("example"),
        s if s.starts_with("/websocket/echo")=>Some("ws-echo"),
        s if s.starts_with("/websocket/broadcast")=>Some("ws-broadcast"),
        _=>None,
    }
}

pub async fn call<S:HttpSocket+Sized+Send+'static>(name: &str, shared: &SharedData, middle_data: &MiddlewareData<S::Stream>, path: &str, mut res: S)->HttpResult<()>{
    match name{
        "example"=>example(shared, path, res).await,
        "ws-echo"=>ws_echo(shared, path, res).await,
        "ws-broadcast"=>ws_broadcast(shared, path, res, Arc::clone(&middle_data.clients)).await,
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
            let ws=res.websocket().await?;
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

pub async fn ws_broadcast<S: HttpSocket + Sized + Send + 'static>(
    _shared: &SharedData,
    _path: &str,
    mut res: S,
    clients: SharedClients<S::Stream>,
) -> HttpResult<()> {
    let c = res.get_client().await?;
    match c.headers.get("upgrade").map(|h| h[0].as_str()).as_deref() {
        Some("websocket") => {
            let ws = Arc::new(res.websocket().await?);
            println!("ws-broadcast: client {} connected", ws.addr);
            
            let mut lock = clients.lock().await;
            lock.push(ws.clone());
            drop(lock);

            // Read loop
            loop {
                let frames = ws.incoming().await.unwrap_or(Vec::new());
                if frames.is_empty() {
                    break;
                }

                for frame in frames {
                    println!("ws-broadcast: received {:?} {} bytes",frame.ftype,frame.payload.len());

                    match frame.ftype {
                        WebSocketFrameType::Ping => {
                            ws.send_pong(frame.get_payload()).await?;
                        }
                        WebSocketFrameType::Text | WebSocketFrameType::Binary => {
                            let payload = frame.get_payload();
                            let mut dead_clients = vec![];

                            let mut lock = clients.lock().await;
                            for (i, client) in lock.iter().enumerate() {
                                if client.addr != ws.addr {
                                    if client.send_text(payload).await.is_err() {
                                        dead_clients.push(i);
                                    }
                                }
                            }

                            for i in dead_clients.into_iter().rev() {
                                lock.remove(i);
                            }
                        }
                        _ => {}
                    }
                }
            }

            
            let mut lock = clients.lock().await;
            lock.retain(|c| c.addr != ws.addr);
            drop(lock);

            println!("ws-broadcast: client {} disconnected", ws.addr);
            Ok(())
        }
        _ => res.close(b"websocket").await,
    }
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
