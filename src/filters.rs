use crate::handlers;
use crate::models::Db;
use warp::Filter;

pub fn movers(
    token: String,
    db: Db,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    move_users_to_new(token.clone(), db.clone())
        .or(move_users_to_original(token, db))
        .or(hello())
}

pub fn move_users_to_original(
    token: String,
    db: Db,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("api" / "move" / String / "original")
        .and(warp::get())
        .and(warp::any().map(move || token.clone()))
        .and(with_db(db))
        .and_then(handlers::move_users_to_original)
}

pub fn move_users_to_new(
    token: String,
    db: Db,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("api" / "move" / String)
        .and(warp::get())
        .and(warp::any().map(move || token.clone()))
        .and(with_db(db))
        .and_then(handlers::move_users_to_group)
}

pub fn hello() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("api" / "hello" / String).map(|name| format!("Hello, {}!", name))
}

fn with_db(db: Db) -> impl Filter<Extract = (Db,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}
