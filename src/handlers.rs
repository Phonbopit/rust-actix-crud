use super::DbPool;

use actix_web::{delete, get, post, put, web, Error, HttpResponse};
use diesel::prelude::*;

use crate::models::{NewTweet, Tweet, TweetPayload};

type DbError = Box<dyn std::error::Error + Send + Sync>;

#[post("/tweets")]
async fn create(
  pool: web::Data<DbPool>,
  payload: web::Json<TweetPayload>,
) -> Result<HttpResponse, Error> {
  let tweet = web::block(move || {
    let conn = pool.get()?;
    add_a_tweet(&payload.message, &conn)
  })
  .await?
  .map_err(actix_web::error::ErrorInternalServerError)?;

  Ok(HttpResponse::Ok().json(tweet))
}

#[get("/tweets")]
async fn index(pool: web::Data<DbPool>) -> Result<HttpResponse, Error> {
  let tweets = web::block(move || {
    let conn = pool.get()?;
    find_all(&conn)
  })
  .await?
  .map_err(actix_web::error::ErrorInternalServerError)?;

  Ok(HttpResponse::Ok().json(tweets))
}

#[get("/tweets/{id}")]
async fn show(id: web::Path<i32>, pool: web::Data<DbPool>) -> Result<HttpResponse, Error> {
  let tweet = web::block(move || {
    let conn = pool.get()?;
    find_by_id(id.into_inner(), &conn)
  })
  .await?
  .map_err(actix_web::error::ErrorInternalServerError)?;

  Ok(HttpResponse::Ok().json(tweet))
}

#[put("/tweets/{id}")]
async fn update(
  id: web::Path<i32>,
  payload: web::Json<TweetPayload>,
  pool: web::Data<DbPool>,
) -> Result<HttpResponse, Error> {
  let tweet = web::block(move || {
    let conn = pool.get()?;
    update_tweet(id.into_inner(), payload.message.clone(), &conn)
  })
  .await?
  .map_err(actix_web::error::ErrorInternalServerError)?;

  Ok(HttpResponse::Ok().json(tweet))
}

#[delete("/tweets/{id}")]
async fn destroy(id: web::Path<i32>, pool: web::Data<DbPool>) -> Result<HttpResponse, Error> {
  let result = web::block(move || {
    let conn = pool.get()?;
    delete_tweet(id.into_inner(), &conn)
  })
  .await?
  .map(|tweet| HttpResponse::Ok().json(tweet))
  .map_err(actix_web::error::ErrorInternalServerError)?;

  Ok(result)
}

fn add_a_tweet(_message: &str, conn: &PgConnection) -> Result<Tweet, DbError> {
  use crate::schema::tweets::dsl::*;

  let new_tweet = NewTweet {
    message: _message,
    created_at: chrono::Local::now().naive_local(),
  };

  let res = diesel::insert_into(tweets)
    .values(&new_tweet)
    .get_result(conn)?;
  Ok(res)
}

fn find_all(conn: &PgConnection) -> Result<Vec<Tweet>, DbError> {
  use crate::schema::tweets::dsl::*;

  let items = tweets.load::<Tweet>(conn)?;
  Ok(items)
}

fn find_by_id(tweet_id: i32, conn: &PgConnection) -> Result<Option<Tweet>, DbError> {
  use crate::schema::tweets::dsl::*;

  let tweet = tweets
    .filter(id.eq(tweet_id))
    .first::<Tweet>(conn)
    .optional()?;

  Ok(tweet)
}

fn update_tweet(tweet_id: i32, _message: String, conn: &PgConnection) -> Result<Tweet, DbError> {
  use crate::schema::tweets::dsl::*;

  let tweet = diesel::update(tweets.find(tweet_id))
    .set(message.eq(_message))
    .get_result::<Tweet>(conn)?;
  Ok(tweet)
}

fn delete_tweet(tweet_id: i32, conn: &PgConnection) -> Result<usize, DbError> {
  use crate::schema::tweets::dsl::*;

  let count = diesel::delete(tweets.find(tweet_id)).execute(conn)?;
  Ok(count)
}
