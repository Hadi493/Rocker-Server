#[macro_use]
extern crate diesel;

use bcrypt::{hash, verify, DEFAULT_COST};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use diesel::sqlite::SqliteConnection;
use rocket::fs::{FileServer, NamedFile};
use rocket::http::Status;
use rocket::response::status::Custom;
use rocket::serde::json::Json;
use rocket::State;
use rocket::{get, post, routes, launch, Build, Rocket};
use serde::{Deserialize, Serialize};
use std::path::Path;
use chrono::NaiveDateTime;

mod schema;
use schema::{users, posts, comments};

// Database connection pool type
type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

// Models
#[derive(Queryable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = users)]
struct User {
    #[serde(skip_deserializing)]
    id: i32,
    username: String,
    #[serde(skip_serializing)]
    password_hash: String,
    #[serde(skip_deserializing)]
    created_at: NaiveDateTime,
}

#[derive(Deserialize)]
struct UserCredentials {
    username: String,
    password: String,
}

#[derive(Queryable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = posts)]
struct Post {
    #[serde(skip_deserializing)]
    id: i32,
    user_id: i32,
    content: String,
    #[serde(skip_deserializing)]
    created_at: NaiveDateTime,
}

#[derive(Queryable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = comments)]
struct Comment {
    #[serde(skip_deserializing)]
    id: i32,
    post_id: i32,
    user_id: i32,
    content: String,
    #[serde(skip_deserializing)]
    created_at: NaiveDateTime,
}

// Routes
#[get("/")]
async fn index() -> Option<NamedFile> {
    NamedFile::open("static/index.html").await.ok()
}

#[post("/register", format = "json", data = "<credentials>")]
async fn register(credentials: Json<UserCredentials>, db: &State<DbPool>) -> Result<Json<User>, Custom<String>> {
    let mut conn = db.get().map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;

    let hash = hash(credentials.password.as_bytes(), DEFAULT_COST)
        .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;

    let new_user = User {
        id: 0,
        username: credentials.username.clone(),
        password_hash: hash,
        created_at: chrono::Local::now().naive_local(),
    };

    diesel::insert_into(users::table)
        .values(&new_user)
        .execute(&mut conn)
        .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;

    Ok(Json(new_user))
}

#[post("/login", format = "json", data = "<credentials>")]
async fn login(credentials: Json<UserCredentials>, db: &State<DbPool>) -> Result<Json<User>, Custom<String>> {
    let mut conn = db.get().map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;

    let user = users::table
        .filter(users::username.eq(&credentials.username))
        .first::<User>(&mut conn)
        .map_err(|_| Custom(Status::Unauthorized, "Invalid credentials".to_string()))?;

    if !verify(&credentials.password, &user.password_hash)
        .map_err(|e| Custom(Status::InternalServerError, e.to_string()))? {
        return Err(Custom(Status::Unauthorized, "Invalid credentials".to_string()));
    }

    Ok(Json(user))
}

#[post("/posts", format = "json", data = "<post>")]
async fn create_post(post: Json<Post>, db: &State<DbPool>) -> Result<Json<Post>, Custom<String>> {
    let mut conn = db.get().map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;

    diesel::insert_into(posts::table)
        .values(&*post)
        .execute(&mut conn)
        .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;

    Ok(post)
}

#[get("/posts")]
async fn get_posts(db: &State<DbPool>) -> Result<Json<Vec<Post>>, Custom<String>> {
    let mut conn = db.get().map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;

    let posts = posts::table
        .load::<Post>(&mut conn)
        .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;

    Ok(Json(posts))
}

#[post("/posts/<post_id>/comments", format = "json", data = "<comment>")]
async fn create_comment(
    post_id: i32,
    comment: Json<Comment>,
    db: &State<DbPool>,
) -> Result<Json<Comment>, Custom<String>> {
    let mut conn = db.get().map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;

    diesel::insert_into(comments::table)
        .values(&*comment)
        .execute(&mut conn)
        .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;

    Ok(comment)
}

#[launch]
fn rocket() -> Rocket<Build> {
    // Set up database connection pool
    let manager = ConnectionManager::<SqliteConnection>::new("db.sqlite3");
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool");

    let config = rocket::Config::figment()
        .merge(("port", 8088))
        .merge(("address", "0.0.0.0"));

    rocket::build()
        .configure(config)
        .mount("/", routes![
            index,
            register,
            login,
            create_post,
            get_posts,
            create_comment
        ])
        .mount("/static", FileServer::from("static"))
        .manage(pool)
}