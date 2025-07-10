use crate::{structs::SharedData, Http1Socket};

// use std::convert::Infallible;
use std::{
    fs::{self, File}, io::Read, path::{Component, Path, PathBuf}
};

// use crate::mime_map::mime_map;

// use regex::Regex;
use rust_http::traits::HttpSocket;

pub async fn handler(shared: &SharedData, mut req: Http1Socket) -> std::io::Result<()> {
    println!("Serving connection");

    let serve_dir=&shared.serve_dir;

    if let Err(err)=req.update_client().await{
        eprintln!("error at Http1Socket::update_client() \n{:?}",err);
    };

    // let path_cleanup1: Regex = Regex::new(r"/+").unwrap();
    // let path_cleanup2: Regex = Regex::new(r"/$").unwrap();
    // let path_cleanup3: Regex = Regex::new(r"\?*.").unwrap();

    let full_path: String = { 
        let full_path: String = req.client.path.clone();
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


        // let full_path   =full_path.replace("\\", "/");
        // let full_path = path_cleanup1.replace_all(&full_path, "/");
        // let full_path = if full_path.len() > 1 {
        //     path_cleanup2.replace(&full_path, "")
        // } else {
        //     full_path
        // };
        // let full_path = if full_path.len() > 1 { path_cleanup3.replace_all(&full_path, "") } else { full_path };

        serve_dir.to_owned() + &full_path.to_string()
    };
    println!("Full path: {}", full_path);

    
    
    let info_res = fs::metadata(&full_path);
    match info_res{
        Ok(info) => {
            if info.is_file() {
                file_handler(shared, &full_path,req).await
                // let mut file = File::open(&full_path).unwrap();
                // let mut buffer = vec![0; info.len() as usize];
                // file.read_exact(&mut buffer).unwrap();
                // Ok(Response::new(Full::new(Bytes::from(buffer))))
            } else if info.is_dir(){
                dir_handler(shared, req, &full_path).await
                // let dir: fs::ReadDir = fs::read_dir(&full_path).unwrap();
                // let mut file: String = "".to_string();
                //
                // for entry in dir{
                //     let entry = entry.unwrap();
                //     let file_name = entry.file_name().into_string().unwrap();
                //     if file_name.starts_with("index.") || file_name == "index" {
                //         file = full_path.to_owned() + "/" + &file_name;
                //         break;
                //     }
                // }
                //
                // if file==""{
                //     error_handler(409, std::io::Error::new(std::io::ErrorKind::IsADirectory,"Cannot find index file in directory"), req).await
                // } else {
                //     file_handler(&file).await
                // }
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

    // Ok(Response::new(Full::new(Bytes::from("Hello, World!"))))
}

pub async fn error_handler(_shared: &SharedData,code: u16, err: std::io::Error, mut req: Http1Socket) -> std::io::Result<()>{
    eprintln!("Error of status {} occoured\n\x1b[31m{}\x1b[0m",code,err);
    match code {
        404 => {
            println!("404 Not Found: {}", &req.client.path);
            req.status=404;
            req.status_msg="404 Not Found".to_owned();
            req.set_header("Content-Type", "text/plain");
            req.close(b"not found").await?;
            Ok(())
        },
        409 => {
            println!("409 Conflict: {}", &req.client.path);
            req.status=409;
            req.status_msg="409 Conflict".to_owned();
            req.set_header("Content-Type", "text/plain");
            req.close(b"conflict").await?;
            Ok(())
        },
        500 => {
            println!("500 Internal Server Error: {}", &req.client.path);
            req.status=500;
            req.status_msg="500 Internal Server Error".to_owned();
            req.set_header("Content-Type", "text/plain");
            req.close(b"internal server error").await?;
            Ok(())
        },
        _ => {
            println!("{}: {}", code, &req.client.path);
            req.status=code;
            req.status_msg=format!("{}: {}", code, err);
            req.set_header("Content-Type", "text/plain");
            req.close(b"internal server error").await?;
            Ok(())
        }
        
    }
}

pub async fn file_handler(shared: &SharedData, path: &str, mut res: Http1Socket) -> std::io::Result<()> {
    let mime=&shared.mime;
    let mut file = File::open(path).unwrap();
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
        res.set_header("Content-Type", ct);
    } else {
        res.set_header("Content-Type", "application/octet-stream");
    };

    let mut buffer = vec![];
    file.read_to_end(&mut buffer).unwrap();
    res.close(&buffer).await?;
    
    Ok(())
}

pub async fn dir_handler(shared: &SharedData, res: Http1Socket,path: &str) -> std::io::Result<()> {
    let dir: fs::ReadDir = fs::read_dir(&path).unwrap();
    let mut file: String = "".to_string();

    println!("Path is dir {}", path);

    for entry in dir{
        let entry = entry.unwrap();
        let file_name = entry.file_name().into_string().unwrap();
        let file_parts: Vec<&str> = (&path).to_owned().split('/').collect();
        let last_dir = file_parts.last().unwrap_or(&".");
        let meta = entry.metadata().unwrap();
        if !meta.is_file() {
            continue; // Mitigate dirs treated as files
        }

        println!("Directory entry {}\n{:?}",file_name,entry);

        if file_name.starts_with("index.") || file_name == "index" || file_name.starts_with(last_dir) {
            // file = path.to_owned() + "/" + &file_name;
            file = entry.path().to_string_lossy().to_string();
            break;
        }
    }

    println!("File found {}", file);

    if fs::metadata(&file).unwrap().is_file(){
        file_handler(shared,&file,res).await
    } else {
        error_handler(shared,409, std::io::Error::new(std::io::ErrorKind::IsADirectory,"Cannot find index file in directory"), res).await
    }
}


