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
use murmur::{
	etf::{balances::Call, runtime_types::node_template_runtime::RuntimeCall::Balances},
	BlockNumber, MurmurStore,
};
use rocket::http::Status;
use rocket::http::{Cookie, CookieJar};
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use sp_core::crypto::Ss58Codec;
use std::fs::File;
use subxt::utils::{AccountId32, MultiAddress};
use subxt_signer::sr25519::dev;

const SALT: &str = "your-server-side-secret-salt";
const EPHEM_MSK: [u8; 32] = [1; 32];

#[derive(Serialize, Deserialize)]
struct LoginRequest {
	username: String,
	password: String,
}

// #[derive(Serialize)]
// pub struct Payload<CallData> {
// 	pallet_name: String,
// 	call_name: String,
// 	call_data: CallData,
// }

// // Implement the From trait for Payload to convert from TxPayload
// impl<C, D> From<murmur::TxPayload<C>> for Payload<D>
// where
// 	D: for<'a> From<&'a C>,
// {
// 	fn from(tx_payload: murmur::TxPayload<C>) -> Self {
// 		Payload {
// 			pallet_name: tx_payload.pallet_name().to_string(),
// 			call_name: tx_payload.call_name().to_string(),
// 			call_data: tx_payload.call_data().into(),
// 		}
// 	}
// }

// #[derive(Serialize, Clone)]
// struct Create {
// 	root: Vec<u8>,
// 	size: u64,
// 	name: Vec<u8>,
// }

// impl<'a> From<&'a murmur::Create> for Create {
// 	fn from(create: &'a murmur::Create) -> Self {
// 		Create { root: create.root.clone(), size: create.size, name: create.name.0.clone() }
// 	}
// }

// #[derive(Serialize)]
// struct CreateResponse {
// 	payload: Payload<Create>,
// 	store: MurmurStore,
// }

// impl<'r> Responder<'r, 'static> for CreateResponse {
// 	fn respond_to(self, _: &'r Request<'_>) -> rocket::response::Result<'static> {
// 		let json_response =
// 			serde_json::to_string(&self).map_err(|_| Status::InternalServerError)?;
// 		Response::build()
// 			.header(rocket::http::ContentType::JSON)
// 			.sized_body(json_response.len(), Cursor::new(json_response))
// 			.ok()
// 	}
// }

// #[derive(Serialize)]
// struct Proxy {
// 	pub name: Vec<u8>,
// 	pub position: u64,
// 	pub hash: Vec<u8>,
// 	pub ciphertext: Vec<u8>,
// 	pub proof: Vec<Vec<u8>>,
// }

// impl<'a> From<&'a murmur::Proxy> for Proxy {
// 	fn from(proxy: &'a murmur::Proxy) -> Self {
// 		Proxy {
// 			name: proxy.name.0.clone(),
// 			position: proxy.position,
// 			hash: proxy.hash.clone(),
// 			ciphertext: proxy.ciphertext.clone(),
// 			proof: proxy.proof.clone(),
// 		}
// 	}
// }

// #[derive(Serialize)]
// struct ProxyResponse {
// 	payload: Payload<Proxy>,
// }

// impl<'r> Responder<'r, 'static> for ProxyResponse {
// 	fn respond_to(self, _: &'r Request<'_>) -> rocket::response::Result<'static> {
// 		let json_response =
// 			serde_json::to_string(&self).map_err(|_| Status::InternalServerError)?;
// 		Response::build()
// 			.header(rocket::http::ContentType::JSON)
// 			.sized_body(json_response.len(), Cursor::new(json_response))
// 			.ok()
// 	}
// }

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
	let seed = derive_seed(username, password);

	cookies.add(Cookie::new("username", username.clone()));
	cookies.add(Cookie::new("seed", seed.clone()));

	"User logged in, session started."
}

#[post("/new", data = "<request>")]
/// Generate a wallet valid for the next {validity} blocks
async fn new(cookies: &CookieJar<'_>, request: Json<NewRequest>) -> Result<String, Status> {
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
			username.to_string(),
			seed.to_string(),
			EPHEM_MSK, // TODO: replace with an hkdf?
			schedule,
			round_pubkey_bytes,
		);
		// 3. add to storage
		write_mmr_store(mmr_store.clone());
		// sign and send the call
		let from = dev::alice();
		let _events = client
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
async fn execute(cookies: &CookieJar<'_>, request: Json<ExecuteRequest>) -> Result<String, Status> {
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

		let store: MurmurStore = load_mmr_store();
		let target_block_number: BlockNumber = current_block_number + 1;
		println!("💾 Recovered Murmur store from local file");
		let tx = murmur::prepare_execute(
			username.to_string(),
			seed.to_string(),
			target_block_number,
			store,
			balance_transfer_call,
		)
		.await;

		// submit the tx using alice to sign it
		let _ = client.tx().sign_and_submit_then_watch_default(&tx, &dev::alice()).await;

		Ok("Transaction executed".to_string())
	})
	.await
}

async fn check_cookie<'a, F, Fut, R>(cookies: &'a CookieJar<'_>, callback: F) -> Result<R, Status>
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

fn derive_seed(password: &str, username: &str) -> String {
	hash_with_salt(format!("{}:{}", username, password), 4, SALT.as_bytes())
		.unwrap()
		.to_string()
}

fn load_mmr_store() -> MurmurStore {
	// TODO: load from DB
	let mmr_store_file = File::open("mmr_store").expect("Unable to open file");
	let data: MurmurStore = serde_cbor::from_reader(mmr_store_file).unwrap();

	data
}
/// Write the MMR data to a file
fn write_mmr_store(mmr_store: MurmurStore) {
	// TODO: write to DB
	let mmr_store_file = File::create("mmr_store").expect("It should create the file");
	serde_cbor::to_writer(mmr_store_file, &mmr_store).unwrap();
}

#[launch]
fn rocket() -> _ {
	rocket::build().mount("/", routes![login, new, execute])
}
