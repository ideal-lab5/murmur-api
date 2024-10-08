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

use rocket::State;
use store::Store;

use murmur::{
	etf::{balances::Call, runtime_types::node_template_runtime::RuntimeCall::Balances},
	BlockNumber,
};
use rocket::{
	http::{Cookie, CookieJar, Status},
	serde::{json::Json, Deserialize},
};
use rocket_db_pools::mongodb::bson::doc;
use sp_core::crypto::Ss58Codec;
use std::env;
use subxt::utils::{AccountId32, MultiAddress};
use subxt_signer::sr25519::dev;
use utils::{check_cookie, derive_seed};

fn get_salt() -> String {
	env::var("SALT").unwrap_or_else(|_| "0123456789abcdef".to_string())
}

fn get_ephem_msk() -> [u8; 32] {
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

#[derive(Deserialize)]
struct LoginRequest {
	username: String,
	password: String,
}

#[derive(Deserialize)]
struct ExecuteRequest {
	amount: u128,
	to: String,
}

#[derive(Deserialize)]
struct NewRequest {
	validity: u32,
}

#[post("/login", data = "<login_request>")]
async fn login(login_request: Json<LoginRequest>, cookies: &CookieJar<'_>) -> &'static str {
	let username = &login_request.username;
	let password = &login_request.password;
	let seed = derive_seed(username, password, &get_salt());

	cookies.add(Cookie::new("username", username.clone()));
	cookies.add(Cookie::new("seed", seed.clone()));

	"User logged in, session started."
}

#[post("/new", data = "<request>")]
/// Generate a wallet valid for the next {validity} blocks
async fn new(
	cookies: &CookieJar<'_>,
	request: Json<NewRequest>,
	db: &State<Store>,
) -> Result<String, Status> {
	check_cookie(cookies, |username, seed| async {
		let (client, current_block_number, round_pubkey_bytes) =
			murmur::idn_connect().await.map_err(|_| Status::InternalServerError)?;

		// 1. prepare block schedule
		let mut schedule: Vec<BlockNumber> = Vec::new();
		for i in 2..request.validity + 2 {
			// wallet is 'active' in 2 blocks
			let next_block_number: BlockNumber = current_block_number + i;
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
		.map_err(|_| Status::InternalServerError)?;

		// 3. add to storage
		let username_string: String = username.into();
		db.write(username_string, mmr_store.clone(), None)
			.await
			.map_err(|_| Status::InternalServerError)?;

		// sign and send the call
		let from = dev::alice();
		client
			.tx()
			.sign_and_submit_then_watch_default(&call, &from)
			.await
			.map_err(|_| Status::InternalServerError)?;
		Ok("MMR proxy account creation successful!".to_string())
	})
	.await
}

#[post("/execute", data = "<request>")]
/// Execute a transaction from the wallet
async fn execute(
	cookies: &CookieJar<'_>,
	request: Json<ExecuteRequest>,
	db: &State<Store>,
) -> Result<String, Status> {
	check_cookie(cookies, |username, seed| async {
		let (client, current_block_number, _) =
			murmur::idn_connect().await.map_err(|_| Status::InternalServerError)?;

		let from_ss58 = sp_core::crypto::AccountId32::from_ss58check(&request.to).unwrap();

		let bytes: &[u8] = from_ss58.as_ref();
		let from_ss58_sized: [u8; 32] = bytes.try_into().unwrap();
		let to = AccountId32::from(from_ss58_sized);
		let balance_transfer_call = Balances(Call::transfer_allow_death {
			dest: MultiAddress::<_, u32>::from(to),
			value: request.amount,
		});

		let username_string = username.into();
		let mmr_option =
			db.load(username_string, None).await.map_err(|_| Status::InternalServerError)?;

		let murmur_store = mmr_option.ok_or(Status::BadRequest)?;

		let target_block_number = current_block_number + 1;

		let tx = murmur::prepare_execute(
			username.into(),
			seed.into(),
			target_block_number,
			murmur_store,
			balance_transfer_call,
		)
		.map_err(|_| Status::InternalServerError)?;

		// submit the tx using alice to sign it
		client
			.tx()
			.sign_and_submit_then_watch_default(&tx, &dev::alice())
			.await
			.map_err(|_| Status::InternalServerError)?;
		Ok("Transaction executed".to_string())
	})
	.await
}

#[launch]
async fn rocket() -> _ {
	let store = Store::init().await;
	rocket::build().mount("/", routes![login, new, execute]).manage(store)
}
