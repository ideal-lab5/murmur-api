#[macro_use]
extern crate rocket;

#[macro_use]
extern crate serde_json;

use bcrypt::{hash, verify};
use rand::Rng;
use rocket::http::{Cookie, CookieJar};
use rocket::serde::json::Json;
use rocket::State;
// use rocket_session_store::SessionStore;
use rocket::http::Status;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

const SALT: &str = "your-server-side-secret-salt";

#[derive(Serialize, Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Default)]
struct Session {
    username: String,
    seed: String,
}

#[post("/login", data = "<login_request>")]
async fn login(
    login_request: Json<LoginRequest>,
    cookies: &CookieJar<'_>,
    // session_store: &State<SessionStore>,
) -> &'static str {
    let username = &login_request.username;
    let password = &login_request.password;
    let seed = derive_seed(username, password);

    cookies.add(Cookie::new("username", username.clone()));
    cookies.add(Cookie::new("seed", seed.clone()));
    // cookies.add_private(Cookie::new("seed", seed.clone()));

    // let mut sessions = session_store.sessions.lock().unwrap();
    // sessions.insert(username.clone(), seed);

    "User logged in, session started."
}

#[post("/create")]
async fn create(cookies: &CookieJar<'_>) -> Result<String, Status> {
    check_cookie(cookies, |_username, _seed| {
        // the cookie is valid and we have the username and seed
        // now we need to validate the inputs
        // murmur.create(seed, uname, 10000, injectedSigner);
        "create mmr called".to_string()
    })
    .map_err(|_| Status::Forbidden)
}

fn check_cookie(
    cookies: &CookieJar<'_>,
    // session_store: &State<SessionStore>,
    callback: fn(username: &str, seed: &str) -> String,
) -> Result<String, ()> {
    let username = cookies.get("username");
    let seed = cookies.get("seed");
    match (username, seed) {
        (Some(username_cookie), Some(seed_cookie)) => {
            let username = username_cookie.value();
            let seed = seed_cookie.value();
            Ok(callback(username, seed))
        }
        _ => Err(()),
    }
}

fn derive_seed(password: &str, username: &str) -> String {
    hash(format!("{}:{}{}", username, password, SALT), 4).unwrap()
}

#[launch]
fn rocket() -> _ {
    // Instance a store that fits your needs and wrap it in a Box in SessionStore.
    // let memory_store: MemoryStore<Session> = Session {
    //     username: MemoryStore::default(),
    //     seed: MemoryStore::default(),
    // };
    // let store: SessionStore<Session> = SessionStore {
    //     store: Box::new(memory_store),
    //     name: "murmur".into(),
    //     duration: Duration::from_secs(3600 * 24 * 3),
    //     // The cookie config is used to set the cookie's path and other options.
    //     cookie: CookieConfig::default(),
    // };
    rocket::build()
        // .attach(store.fairing())
        .mount("/", routes![login, create])
}
