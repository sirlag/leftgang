use crate::handlers;
use sqlx::{Pool, Postgres};
use warp::Filter;

pub fn movers(
    token: String,
    pool: Pool<Postgres>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    move_users_to_new(token.clone(), pool.clone())
        .or(move_users_to_original(token, pool))
        .or(hello())
}

pub fn move_users_to_original(
    token: String,
    pool: Pool<Postgres>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("api" / "move" / String / "original")
        .and(warp::get())
        .and(warp::any().map(move || token.clone()))
        .and(warp::any().map(move || pool.clone()))
        .and_then(handlers::move_users_to_original)
}

pub fn move_users_to_new(
    token: String,
    pool: Pool<Postgres>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("api" / "move" / String)
        .and(warp::get())
        .and(warp::any().map(move || token.clone()))
        .and(warp::any().map(move || pool.clone()))
        .and_then(handlers::move_users_to_group)
}

pub fn hello() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("api" / "hello" / String).map(|name| format!("Hello, {}!", name))
}
