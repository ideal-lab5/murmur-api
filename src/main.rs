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
mod utils;

use murmur::{
	etf::{balances::Call, runtime_types::node_template_runtime::RuntimeCall::Balances},
	BlockNumber,
};
use rocket::http::Method;
use rocket::http::Status;
use rocket::http::{Cookie, CookieJar};
use rocket::serde::{json::Json, Deserialize};
use rocket_cors::{AllowedOrigins, CorsOptions};
use sp_core::crypto::Ss58Codec;
use subxt::utils::{AccountId32, MultiAddress};
use subxt_signer::sr25519::dev;
use utils::{check_cookie, derive_seed, get_ephem_msk, get_salt, MurmurError};

#[derive(Deserialize)]
struct AuthRequest {
	username: String,
	password: String,
}

#[derive(Deserialize)]
struct ExecuteRequest {
	amount: String,
	to: String,
	current_block_number: BlockNumber,
}

#[derive(Deserialize)]
struct NewRequest {
	validity: u32,
	current_block_number: BlockNumber,
	round_pubkey_bytes: String,
}

#[post("/authenticate", data = "<auth_request>")]
/// Authenticate the user and start a session
async fn authenticate(auth_request: Json<AuthRequest>, cookies: &CookieJar<'_>) -> &'static str {
	let username = &auth_request.username;
	let password = &auth_request.password;
	let seed = derive_seed(username, password, &get_salt());

	cookies.add(Cookie::new("username", username.clone()));
	cookies.add(Cookie::new("seed", seed.clone()));

	"User authenticated, session started."
}

#[post("/new", data = "<request>")]
/// Generate a wallet valid for the next {request.validity} blocks
async fn new(
	cookies: &CookieJar<'_>,
	request: Json<NewRequest>,
) -> Result<String, (Status, String)> {
	check_cookie(cookies, |username, seed| async {
		let (client, _, _) = murmur::idn_connect()
			.await
			.map_err(|e| (Status::InternalServerError, e.to_string()))?;

		let round_pubkey_bytes = hex::decode(request.round_pubkey_bytes.clone()).map_err(|_| {
			(
				Status::BadRequest,
				format!(
					"There is an issue with the `round_pubkey_bytes` input `{:?}`",
					&request.round_pubkey_bytes
				),
			)
		})?;
		// 1. prepare block schedule
		let mut schedule: Vec<BlockNumber> = Vec::new();
		for i in 2..request.validity + 2 {
			// wallet is 'active' in 2 blocks
			let next_block_number: BlockNumber = request.current_block_number + i;
			schedule.push(next_block_number);
		}
		// 2. create mmr
		let (call, mmr_store) = murmur::create(
			username.into(),
			seed.into(),
			get_ephem_msk(), // TODO: replace with an hkdf? https://github.com/ideal-lab5/murmur/issues/13
			schedule,
			round_pubkey_bytes,
		)
		.map_err(|e| (Status::InternalServerError, MurmurError(e).to_string()))?;
		// 3. add to storage
		store::write(mmr_store.clone());
		// sign and send the call
		let from = dev::alice();
		let _events = client
			.tx()
			.sign_and_submit_then_watch_default(&call, &from)
			.await
			.map_err(|e| (Status::InternalServerError, e.to_string()))?;
		Ok("MMR proxy account creation successful!".to_string())
	})
	.await
}

#[post("/execute", data = "<request>")]
/// Execute a transaction from the wallet
async fn execute(
	cookies: &CookieJar<'_>,
	request: Json<ExecuteRequest>,
) -> Result<String, (Status, String)> {
	check_cookie(cookies, |username, seed| async {
		let (client, _, _) = murmur::idn_connect()
			.await
			.map_err(|e| (Status::InternalServerError, e.to_string()))?;

		let from_ss58 =
			sp_core::crypto::AccountId32::from_ss58check(&request.to).map_err(|_| {
				(
					Status::BadRequest,
					format!("There is an issue with the `to` input `{:?}`", &request.to),
				)
			})?;
		let bytes: &[u8] = from_ss58.as_ref();
		let from_ss58_sized: [u8; 32] = bytes.try_into().map_err(|_| {
			(
				Status::BadRequest,
				format!("There is an issue with the `to` input `{:?}`", &request.to),
			)
		})?;
		let to = AccountId32::from(from_ss58_sized);

		let value = request.amount.parse::<u128>().map_err(|_| {
			(
				Status::BadRequest,
				format!("There is an issue with the `amount` input `{:?}`", &request.amount),
			)
		})?;

		let balance_transfer_call =
			Balances(Call::transfer_allow_death { dest: MultiAddress::<_, u32>::from(to), value });

		let store = store::load();
		let target_block_number = request.current_block_number + 1;

		let tx = murmur::prepare_execute(
			username.into(),
			seed.into(),
			target_block_number,
			store,
			balance_transfer_call,
		)
		.map_err(|e| (Status::InternalServerError, MurmurError(e).to_string()))?;

		// submit the tx using alice to sign it
		let _ = client.tx().sign_and_submit_then_watch_default(&tx, &dev::alice()).await;

		Ok("Transaction executed".to_string())
	})
	.await
}

#[launch]
fn rocket() -> _ {
	let cors = CorsOptions::default()
		.allowed_origins(AllowedOrigins::all())
		.allowed_methods(
			vec![Method::Get, Method::Post, Method::Patch]
				.into_iter()
				.map(From::from)
				.collect(),
		)
		.allow_credentials(true)
		.to_cors()
		.unwrap();

	rocket::build().mount("/", routes![authenticate, new, execute]).attach(cors)
}
