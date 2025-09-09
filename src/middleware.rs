use rust_http::common::{HttpError, HttpResult, HttpSocket};

use crate::structs::SharedData;
// use std::{collections::HashMap};


pub fn available(path: &str)->Option<&'static str>{
    match path{
        s if s.starts_with("/internal/example")=>Some("example"),
        _=>None,
    }
}

pub async fn call<S:HttpSocket>(name: &str, shared: &SharedData, path: &str, mut res: S)->HttpResult<()>{
    match name{
        "example"=>example(shared, path, res).await,
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
