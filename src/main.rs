#[macro_use]
extern crate rocket;

use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

// Define data structures
#[derive(Serialize, Deserialize)]
struct Message {
    content: String,
}

#[derive(Serialize, Deserialize)]
struct User {
    id: u32,
    name: String,
    email: String,
}

// Route handlers
#[get("/")]
fn index() -> &'static str {
    "Hello, welcome to Rocket API! it's my first Rocket server"
}

#[get("/user/<id>")]
fn get_user(id: u32) -> Json<User> {
    let user = User {
        id,
        name: String::from("John Doe"),
        email: String::from("john@example.com"),
    };
    Json(user)
}

#[post("/message", data = "<message>")]
fn create_message(message: Json<Message>) -> Json<Message> {
    message
}

#[get("/hello?<name>")]
fn hello(name: Option<String>) -> String {
    match name {
        Some(name) => format!("Hello, {}!", name),
        None => String::from("Hello, world!"),
    }
}

#[catch(404)]
fn not_found() -> &'static str {
    "Resource not found!"
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index, get_user, create_message, hello])
        .register("/", catchers![not_found])
}
