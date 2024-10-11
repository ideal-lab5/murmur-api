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

mod store;
mod translate;
mod types;
mod utils;

use dotenv::dotenv;
use murmur::{BlockNumber, RuntimeCall};
use parity_scale_codec::Decode;
use rocket::{
	http::{Cookie, CookieJar, Method, SameSite, Status},
	serde::json::Json,
	State,
};
use rocket_cors::{AllowedHeaders, AllowedOrigins, CorsOptions};
use rocket_db_pools::mongodb::bson::doc;
use std::env;
use store::Store;
use types::{AuthRequest, CreateRequest, CreateResponse, ExecuteRequest, ExecuteResponse};
use utils::{check_cookie, derive_seed, MurmurError};

#[post("/authenticate", data = "<auth_request>")]
/// Authenticate the user and start a session
async fn authenticate(auth_request: Json<AuthRequest>, cookies: &CookieJar<'_>) -> &'static str {
	let username = &auth_request.username;
	let password = &auth_request.password;
	let seed = derive_seed(username, password, &env::var("SALT").unwrap());

	let username_cookie = Cookie::build(("username", username.clone()))
		.path("/")
		.same_site(SameSite::None)
		.secure(true);

	let seed_cookie = Cookie::build(("seed", seed.clone()))
		.path("/")
		.same_site(SameSite::None)
		.secure(true);

	cookies.add(username_cookie);
	cookies.add(seed_cookie);

	"User authenticated, session started."
}

#[post("/create", data = "<request>")]
/// Prepare and return the data needed to create a wallet
/// valid for the next {request.validity} blocks
async fn create(
	cookies: &CookieJar<'_>,
	request: Json<CreateRequest>,
	db: &State<Store>,
) -> Result<CreateResponse, (Status, String)> {
	check_cookie(cookies, |username, seed| async {
		let round_pubkey_bytes = translate::pubkey_to_bytes(&request.round_pubkey)
			.map_err(|e| (Status::BadRequest, format!("`request.round_pubkey`: {:?}", e)))?;

		// 1. prepare block schedule
		let mut schedule: Vec<BlockNumber> = Vec::new();
		for i in 2..request.validity + 2 {
			// wallet is 'active' in 2 blocks
			let next_block: BlockNumber = request.current_block + i;
			schedule.push(next_block);
		}

		// 2. create mmr
		let create_data =
			murmur::create(seed.into(), request.ephem_msk, schedule, round_pubkey_bytes)
				.map_err(|e| (Status::InternalServerError, MurmurError(e).to_string()))?;

		// 3. add to storage
		db.write(username.into(), create_data.mmr_store.clone())
			.await
			.map_err(|e| (Status::InternalServerError, e.to_string()))?;

		// 4. return the call data
		Ok(CreateResponse { create_data, username: username.into() })
	})
	.await
}

#[post("/execute", data = "<request>")]
/// Execute a transaction from the wallet
async fn execute(
	cookies: &CookieJar<'_>,
	request: Json<ExecuteRequest>,
	db: &State<Store>,
) -> Result<ExecuteResponse, (Status, String)> {
	check_cookie(cookies, |username, seed| async {
		let mmr_option = db
			.load(username)
			.await
			.map_err(|e| (Status::InternalServerError, e.to_string()))?;

		let store = mmr_option
			.ok_or((Status::InternalServerError, "No Murmur Store for username".to_string()))?;
		let target_block = request.current_block + 1;

		let call = RuntimeCall::decode(&mut &request.runtime_call[..])
			.map_err(|e| (Status::InternalServerError, e.to_string()))?;

		let proxy_data = murmur::prepare_execute(seed.into(), target_block, store, &call)
			.map_err(|e| (Status::InternalServerError, MurmurError(e).to_string()))?;

		Ok(ExecuteResponse { username: username.into(), proxy_data })
	})
	.await
}

#[launch]
async fn rocket() -> _ {
	dotenv().ok();
	let store = Store::init().await;
	let cors = CorsOptions::default()
		.allowed_origins(AllowedOrigins::all())
		.allowed_methods(
			vec![Method::Get, Method::Post, Method::Patch]
				.into_iter()
				.map(From::from)
				.collect(),
		)
		.allowed_headers(AllowedHeaders::all())
		.allow_credentials(true)
		.to_cors()
		.unwrap();

	rocket::build()
		.mount("/", routes![authenticate, create, execute])
		.manage(store)
		.attach(cors)
}
