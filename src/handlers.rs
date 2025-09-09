use crate::{middleware, structs::SharedData /*Http1Socket*/};

use tokio::{
    fs::{self, File}, io::AsyncReadExt,
};
use std::{
    // io::Read, 
    path::{Component, Path, PathBuf}
};

use rust_http::common::{Compression, HttpClient, HttpResult, HttpSocket /*HttpConstructor, Stream*/};

pub async fn handler<S:HttpSocket>(shared: &SharedData, mut req: S) -> HttpResult<()> {
    println!("Serving connection");

    let serve_dir=&shared.serve_dir;

    let client=match req.get_client().await{
        Err(err)=>{
            eprintln!("error at Http1Socket::update_client() \n{:?}",err);
            HttpClient::empty()
        },
        Ok(c)=>c.clone(),
    };

    // dbg!(&client);

    if let Some(ae)=client.headers.get("accept-encoding"){
        let s=ae.join(" ");
        if s.contains("gzip"){ 
            req.set_compression(Compression::Gzip).unwrap();  
        }
    }

    let full_path: String = { 
        let full_path: String = client.path.clone();
        let full_path = full_path.replace("\\","/");
        let full_path = full_path.split(|c| c == '?' || c == '#').next().unwrap_or("");
        
        let mut cleaned = PathBuf::new();
        for comp in Path::new(full_path).components() {
            match comp {
                Component::Normal(s) => cleaned.push(s),
                Component::RootDir => cleaned.push("/"),
                _ => {} // Skip CurDir, ParentDir, Prefix, etc.
            }
        }

        let full_path = cleaned.to_string_lossy().into_owned();
        let full_path = full_path.replace("\\","/");


        serve_dir.to_owned() + &full_path.to_string()
    };
    println!("Full path: {}", &full_path);


    if let Some(n)=middleware::available(&client.path){
        println!("Middleware available: {}", n);
        
        match middleware::call(n, shared, &full_path, req).await{
            Ok(_)=>println!("middleware did not error"),
            Err(e)=>eprintln!("\x1b[31mmiddleware errored\x1b[0m {}",e),
        };
        Ok(())
    } else {
        let info_res = fs::metadata(&full_path).await;
        match info_res{
            Ok(info) => {
                if info.is_file() {
                    file_handler(shared, &full_path,req).await
                } else if info.is_dir(){
                    dir_handler(shared, req, &full_path).await
                } else {
                    error_handler(shared, 409, std::io::Error::new(std::io::ErrorKind::Unsupported, "File is unusable"), req).await
                }
            },
            Err(err) => {
                if err.kind() == std::io::ErrorKind::NotFound {
                    error_handler(shared,404, err, req).await
                } else {
                    error_handler(shared,500, err, req).await
                }
            },
        }
    }
}

pub async fn error_handler<S:HttpSocket>(_shared: &SharedData,code: u16, err: std::io::Error, mut req: S) -> HttpResult<()>{
    eprintln!("Error of status {} occoured\n\x1b[31m{}\x1b[0m",code,err);
    match code {
        404 => {
            println!("404 Not Found: {}", &req.get_client().await?.path);
            req.set_status(404, "Not found".to_owned())?;
            // req.status_msg="404 Not Found".to_owned();
            let _=req.set_header("Content-Type", "text/plain");
            req.close(b"not found").await?;
            Ok(())
        },
        409 => {
            println!("409 Conflict: {}", &req.get_client().await?.path);
            req.set_status(409, "Conflict".to_owned())?;
            // req.status=409;
            // req.status_msg="409 Conflict".to_owned();
            let _=req.set_header("Content-Type", "text/plain");
            req.close(b"conflict").await?;
            Ok(())
        },
        500 => {
            println!("500 Internal Server Error: {}", &req.get_client().await?.path);
            req.set_status(500, "Internal Server Error".to_owned())?;
            // req.status=500;
            // req.status_msg="500 Internal Server Error".to_owned();
            let _=req.set_header("Content-Type", "text/plain");
            req.close(b"internal server error").await?;
            Ok(())
        },
        _ => {
            println!("{}: {}", code, &req.get_client().await?.path);
            req.set_status(code, format!("{}: {}", code, err))?;
            // req.status=code;
            // req.status_msg=format!("{}: {}", code, err);
            let _=req.set_header("Content-Type", "text/plain");
            req.close(b"internal server error").await?;
            Ok(())
        }
        
    }
}

pub async fn file_handler<S:HttpSocket>(shared: &SharedData, path: &str, mut res: S) -> HttpResult<()> {
    let mime=&shared.mime;
    let mut file = File::open(path).await.unwrap();
    // let mut buffer = vec![];
    let parts: Vec<&str>=path.split(".").collect::<Vec<&str>>();
    let last=parts[parts.len()-1];

    /*if path.ends_with(".html"){
        res.set_header("Content-Type", "text/html");
    } else if path.ends_with(".css") {
        res.set_header("Content-Type", "text/css");
    } else if path.ends_with(".js") {
        res.set_header("Content-Type", "application/javascript");
    } else if path.ends_with(".json") {
        res.set_header("Content-Type", "application/json");
    } else if path.ends_with(".png") {
        res.set_header("Content-Type", "image/png");
    } else if path.ends_with(".jpg") || path.ends_with(".jpeg") {
        res.set_header("Content-Type", "image/jpeg");
    } else if path.ends_with(".gif") {
        res.set_header("Content-Type", "image/gif");
    } else*/ if let Some(ct)=mime.get(last) {
        let _=res.set_header("Content-Type", ct);
    } else {
        let _=res.set_header("Content-Type", "application/octet-stream");
    };

    let mut buffer = vec![];
    file.read_to_end(&mut buffer).await.unwrap();
    res.close(&buffer).await?;
    
    Ok(())
}

pub async fn dir_handler<S:HttpSocket>(shared: &SharedData, res: S,path: &str) -> HttpResult<()> {
    let mut dir = fs::read_dir(&path).await?;
    let mut file: String = "".to_string();

    println!("Path is dir {}", path);

    while let Some(entry)=dir.next_entry().await?{
        // let entry = entry.unwrap();
        let file_name = entry.file_name().into_string().unwrap_or("unknown".to_owned());
        let file_parts: Vec<&str> = (&path).to_owned().split('/').collect();
        let last_dir = file_parts.last().unwrap_or(&".");
        let meta = entry.metadata().await?;
        if !meta.is_file() {
            continue; // Mitigate dirs treated as files
        }

        // println!("Directory entry {}\n{:?}",file_name,entry);

        if file_name.starts_with("index.") || file_name == "index" || (!last_dir.is_empty() && file_name.starts_with(last_dir)) {
            // file = path.to_owned() + "/" + &file_name;
            // println!("last dir = {:?}\npath = {path:?}",last_dir);
            file = entry.path().to_string_lossy().to_string();
            break;
        }
    }

    println!("File found {}", file);

    if fs::metadata(&file).await.unwrap().is_file(){
        file_handler(shared,&file,res).await
    } else {
        error_handler(shared,409, std::io::Error::new(std::io::ErrorKind::IsADirectory,"Cannot find index file in directory"), res).await
    }
}


