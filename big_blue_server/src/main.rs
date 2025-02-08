use warp::{Filter, reply::Response};
use std::{collections::HashMap, fs::read, fs::read_to_string, sync::{Arc, Mutex}};
use big_blue_server::rewards::Reward;
#[tokio::main]
async fn main() {
    let directory = std::env::current_dir().unwrap().to_str().unwrap().to_owned();
    let accounts = read_to_string(directory.clone()+"/accounts.json").unwrap();
    let user_data: Arc<Mutex<HashMap<String, big_blue_server::Account>>> = Arc::new(
        Mutex::new(
            serde_json::from_str(
                &accounts
            ).unwrap()
        )
    );
    let filter = warp::get()
        .and(warp::path::full())
        .map(move |path: warp::filters::path::FullPath| {
            println!("{path:?}");
            let mut not_found = false;
            let html = match path.as_str() {
                _ => {
                    read((directory.clone()+&path.as_str()).replace('/',"\\").replace("%20", " ")).unwrap_or_else(|e| {
                        println!("{:?}",e);
                        not_found = true;
                        read(directory.clone()+"/404.html").unwrap()
                    })
                }
            };
            if path.as_str()=="/rewards_data.json" {
                println!("{}", String::from_utf8(html.clone()).unwrap())
            }
            let mut response = Response::new(html.into());
            match path.as_str().split_once('.') {
                Some((_, s @ ("jpg"|"png"))) => {
                    if not_found {
                        response.headers_mut()
                            .insert("Content-Type", str::parse("text/html")
                            .unwrap());
                    } else {
                        response.headers_mut()
                            .insert("Content-Type", str::parse(&("image/".to_owned() + s))
                            .unwrap());
                    }
                }
                Some((_, s)) => {
                    response.headers_mut()
                        .insert("Content-Type", str::parse(&("text/".to_owned() + s))
                        .unwrap());
                }
                None => {}
            }
            response
        });
    let login = {
        let user_data = user_data.clone();
        warp::post()
            .and(warp::filters::body::form())
            .and(warp::path("/login.html"))
            .map(move |form_response: HashMap<String,String>| {
                let mut user_data = user_data.lock().unwrap();
                let username = form_response.get("username".into()).unwrap().to_owned();
                if user_data.contains_key(&username) {
                    //todo: add authentication
                } else {
                    //todo: add error
                }
            })
    };
    let register = {
        let user_data = user_data.clone();
        warp::post()
            .and(warp::filters::body::form())
            .and(warp::path("/register.html"))
            .map(move |form_response: HashMap<String,String>| {
                let mut user_data = user_data.lock().unwrap();
                let username = form_response.get("username".into()).unwrap().to_owned();
                if user_data.contains_key(&username) {
                    //todo: add error
                } else {
                    //todo: add user creation
                }
            })
    };
    warp::serve(filter)
        .run(([127,0,0,1], 7878))
        .await
}