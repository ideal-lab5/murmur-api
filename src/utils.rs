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

use bcrypt::hash_with_salt;
use rocket::http::{CookieJar, Status};

pub(crate) async fn check_cookie<'a, F, Fut, R>(
	cookies: &'a CookieJar<'_>,
	callback: F,
) -> Result<R, Status>
where
	F: FnOnce(&'a str, &'a str) -> Fut,
	Fut: std::future::Future<Output = Result<R, Status>>,
{
	let username = cookies.get("username");
	let seed = cookies.get("seed");
	match (username, seed) {
		(Some(username_cookie), Some(seed_cookie)) => {
			let username = username_cookie.value();
			let seed = seed_cookie.value();
			callback(username, seed).await
		},
		_ => Err(Status::Forbidden),
	}
}

pub(crate) fn derive_seed(password: &str, username: &str, salt: &str) -> String {
	hash_with_salt(format!("{}:{}", username, password), 4, salt.as_bytes())
		.unwrap()
		.to_string()
}
