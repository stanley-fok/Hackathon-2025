use warp::{Filter, reply::Response};
use std::fs::read;
use big_blue_server::rewards::Reward;
#[tokio::main]
async fn main() {
    let directory =std::env::current_dir().unwrap().to_str().unwrap().to_owned();
    let filter = warp::any()
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
    warp::serve(filter)
        .run(([127,0,0,1], 7878))
        .await
}