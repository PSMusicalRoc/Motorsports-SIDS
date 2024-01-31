use warp::Filter;

use pretty_env_logger as pretty_log;
#[macro_use] extern crate log;

use warp;

#[tokio::main]
async fn main() {
    pretty_log::init();
    
    trace!("Hello world!");

    let index = warp::get()
        .and(warp::path!())
        .and(warp::fs::file("html/index.html"));

    let static_dir = warp::path("static")
        .and(warp::fs::dir("html/static"));

    let routes = index
        .or(static_dir);

    warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;
}
