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
use std::env;

pub(crate) async fn check_cookie<'a, F, Fut, R>(
	cookies: &'a CookieJar<'_>,
	callback: F,
) -> Result<R, (Status, String)>
where
	F: FnOnce(&'a str, &'a str) -> Fut,
	Fut: std::future::Future<Output = Result<R, (Status, String)>>,
{
	let username = cookies.get("username");
	let seed = cookies.get("seed");
	match (username, seed) {
		(Some(username_cookie), Some(seed_cookie)) => {
			let username = username_cookie.value();
			let seed = seed_cookie.value();
			callback(username, seed).await
		},
		_ => Err((Status::Forbidden, "Not logged in".to_string())),
	}
}

pub(crate) fn derive_seed(password: &str, username: &str, salt: &str) -> String {
	hash_with_salt(format!("{}:{}", username, password), 4, salt.as_bytes())
		.unwrap()
		.to_string()
}

pub(crate) struct MurmurError(pub(crate) murmur::Error);

impl std::fmt::Display for MurmurError {
	#[allow(unreachable_patterns)]
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self.0 {
			murmur::Error::ExecuteError => write!(f, "Murmur: Execute error"),
			murmur::Error::MMRError => write!(f, "Murmur: MMR error"),
			murmur::Error::InconsistentStore => write!(f, "Murmur: Inconsistent store"),
			murmur::Error::NoLeafFound => write!(f, "Murmur: No leaf found"),
			murmur::Error::NoCiphertextFound => write!(f, "Murmur: No ciphertext found"),
			murmur::Error::TlockFailed => write!(f, "Murmur: Tlock failed"),
			murmur::Error::InvalidBufferSize => write!(f, "Murmur: Invalid buffer size"),
			murmur::Error::InvalidSeed => write!(f, "Murmur: Invalid seed"),
			murmur::Error::InvalidPubkey => write!(f, "Murmur: Invalid pubkey"),
			_ => write!(f, "Murmur: Unknown error"),
		}
	}
}

pub(crate) fn get_salt() -> String {
	env::var("SALT").unwrap_or_else(|_| "0123456789abcdef".to_string())
}

pub(crate) fn get_ephem_msk() -> [u8; 32] {
	let ephem_msk_str = env::var("EPHEM_MSK").unwrap_or_else(|_| {
		"1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1".to_string()
	});
	let ephem_msk_vec: Vec<u8> = ephem_msk_str
		.split(',')
		.map(|s| s.trim().parse().expect("Invalid integer in EPHEM_MSK"))
		.collect();
	let mut ephem_msk = [0u8; 32];
	for (i, &byte) in ephem_msk_vec.iter().enumerate().take(32) {
		ephem_msk[i] = byte;
	}
	ephem_msk
}
