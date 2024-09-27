#[macro_use]
extern crate rocket;

use bcrypt::hash_with_salt;
use rocket::http::Status;
use rocket::http::{Cookie, CookieJar};
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

const SALT: &str = "your-server-side-secret-salt";

#[derive(Serialize, Deserialize)]
struct LoginRequest {
	username: String,
	password: String,
}

#[post("/login", data = "<login_request>")]
async fn login(login_request: Json<LoginRequest>, cookies: &CookieJar<'_>) -> &'static str {
	let username = &login_request.username;
	let password = &login_request.password;
	let seed = derive_seed(username, password);

	cookies.add(Cookie::new("username", username.clone()));
	cookies.add(Cookie::new("seed", seed.clone()));

	"User logged in, session started."
}

#[post("/create")]
async fn create(cookies: &CookieJar<'_>) -> Result<String, Status> {
	check_cookie(cookies, |_username, _seed| "create mmr called".to_string())
		.map_err(|_| Status::Forbidden)
}

#[post("/execute")]
async fn execute(cookies: &CookieJar<'_>) -> Result<String, Status> {
	check_cookie(cookies, |_username, _seed| "execute called".to_string())
		.map_err(|_| Status::Forbidden)
}

fn check_cookie(
	cookies: &CookieJar<'_>,
	callback: fn(username: &str, seed: &str) -> String,
) -> Result<String, ()> {
	let username = cookies.get("username");
	let seed = cookies.get("seed");
	match (username, seed) {
		(Some(username_cookie), Some(seed_cookie)) => {
			let username = username_cookie.value();
			let seed = seed_cookie.value();
			Ok(callback(username, seed))
		},
		_ => Err(()),
	}
}

fn derive_seed(password: &str, username: &str) -> String {
	hash_with_salt(format!("{}:{}", username, password), 4, SALT.as_bytes())
		.unwrap()
		.to_string()
}

#[launch]
fn rocket() -> _ {
	rocket::build().mount("/", routes![login, create, execute])
}
