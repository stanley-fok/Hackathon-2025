use warp::Filter;
#[tokio::main]
async fn main() {
    let filter = warp::path("home")
        .map(|| {
            let html = std::fs::read("home.html").unwrap();
            warp::reply::Response::new(html.into())
        });
    warp::serve(filter)
        .run(([127,0,0,1], 7878))
        .await
}