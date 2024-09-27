/*
 * Copyright 2024 by Ideal Labs, LLC
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

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
	check_cookie(cookies, |username, seed| async {
		murmur::create(
			username.to_string(),
			seed.to_string(),
			[1; 32],
			vec![1, 2, 3],
			vec![4, 5, 6, 7],
		)
		.await;
		"create mmr called".to_string()
	})
	.await
	.map_err(|_| Status::Forbidden)
}

#[post("/execute")]
async fn execute(cookies: &CookieJar<'_>) -> Result<String, Status> {
	check_cookie(cookies, |_username, _seed| async { "execute called".to_string() })
		.await
		.map_err(|_| Status::Forbidden)
}

async fn check_cookie<'a, F, Fut>(cookies: &'a CookieJar<'_>, callback: F) -> Result<String, ()>
where
	F: FnOnce(&'a str, &'a str) -> Fut,
	Fut: std::future::Future<Output = String>,
{
	let username = cookies.get("username");
	let seed = cookies.get("seed");
	match (username, seed) {
		(Some(username_cookie), Some(seed_cookie)) => {
			let username = username_cookie.value();
			let seed = seed_cookie.value();
			Ok(callback(username, seed).await)
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
