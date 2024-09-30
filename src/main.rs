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
use murmur::etf::runtime_types::node_template_runtime::RuntimeCall;
use murmur::MurmurStore;
use rocket::http::Status;
use rocket::http::{Cookie, CookieJar};
use rocket::serde::json::Json;
use rocket::{response::Responder, Request, Response};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Cursor;
use subxt_signer::sr25519::dev;

const SALT: &str = "your-server-side-secret-salt";

#[derive(Serialize, Deserialize)]
struct LoginRequest {
	username: String,
	password: String,
}

#[derive(Serialize)]
pub struct Payload<CallData> {
	pallet_name: String,
	call_name: String,
	call_data: CallData,
}

// Implement the From trait for Payload to convert from TxPayload
impl<C, D> From<murmur::TxPayload<C>> for Payload<D>
where
	// C: Clone,
	D: for<'a> From<&'a C>,
{
	fn from(tx_payload: murmur::TxPayload<C>) -> Self {
		Payload {
			pallet_name: tx_payload.pallet_name().to_string(),
			call_name: tx_payload.call_name().to_string(),
			call_data: tx_payload.call_data().into(),
		}
	}
}

#[derive(Serialize, Clone)]
struct Create {
	root: Vec<u8>,
	size: u64,
	name: Vec<u8>,
}

impl<'a> From<&'a murmur::Create> for Create {
	fn from(create: &'a murmur::Create) -> Self {
		Create { root: create.root.clone(), size: create.size, name: create.name.0.clone() }
	}
}

#[derive(Serialize)]
struct CreateResponse {
	payload: Payload<Create>,
	store: MurmurStore,
}

impl<'r> Responder<'r, 'static> for CreateResponse {
	fn respond_to(self, _: &'r Request<'_>) -> rocket::response::Result<'static> {
		let json_response =
			serde_json::to_string(&self).map_err(|_| Status::InternalServerError)?;
		Response::build()
			.header(rocket::http::ContentType::JSON)
			.sized_body(json_response.len(), Cursor::new(json_response))
			.ok()
	}
}

#[derive(Serialize)]
struct Proxy {
	pub name: Vec<u8>,
	pub position: u64,
	pub hash: Vec<u8>,
	pub ciphertext: Vec<u8>,
	pub proof: Vec<Vec<u8>>,
}

impl<'a> From<&'a murmur::Proxy> for Proxy {
	fn from(proxy: &'a murmur::Proxy) -> Self {
		Proxy {
			name: proxy.name.0.clone(),
			position: proxy.position,
			hash: proxy.hash.clone(),
			ciphertext: proxy.ciphertext.clone(),
			proof: proxy.proof.clone(),
		}
	}
}

#[derive(Serialize)]
struct ProxyResponse {
	payload: Payload<Proxy>,
}

impl<'r> Responder<'r, 'static> for ProxyResponse {
	fn respond_to(self, _: &'r Request<'_>) -> rocket::response::Result<'static> {
		let json_response =
			serde_json::to_string(&self).map_err(|_| Status::InternalServerError)?;
		Response::build()
			.header(rocket::http::ContentType::JSON)
			.sized_body(json_response.len(), Cursor::new(json_response))
			.ok()
	}
}

#[derive(Deserialize)]
struct PrepareExecuteRequest {
	name: String,
	seed: String,
	amount: u128,
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
async fn create(cookies: &CookieJar<'_>) -> Result<CreateResponse, Status> {
	check_cookie(cookies, |username, seed| async {
		let (create_payload, store) = murmur::create(
			username.to_string(),
			seed.to_string(),
			[1; 32],
			vec![1, 2, 3],
			vec![4, 5, 6, 7],
		);
		CreateResponse { payload: create_payload.into(), store }
	})
	.await
	.map_err(|_| Status::Forbidden)
}

#[post("/prepare_execute", data = "<request>")]
async fn prepare_execute(
	cookies: &CookieJar<'_>,
	request: Json<PrepareExecuteRequest>,
) -> Result<ProxyResponse, Status> {
	check_cookie(cookies, |_username, _seed| async {
		let request = request.into_inner();
		let current_block_number = 1; // TODO: get from runtime
		let bob = dev::bob().public_key();
		let balance_transfer_call =
			RuntimeCall::Balances(murmur::etf::balances::Call::transfer_allow_death {
				dest: subxt::utils::MultiAddress::<_, u32>::from(bob),
				value: request.amount,
			});
		let proxy_payload = murmur::prepare_execute(
			request.name.into_bytes(),
			request.seed.into_bytes(),
			current_block_number,
			load_mmr_store(),
			balance_transfer_call,
		)
		.await;
		ProxyResponse { payload: proxy_payload.into() }
	})
	.await
	.map_err(|_| Status::Forbidden)
}

async fn check_cookie<'a, F, Fut, R>(cookies: &'a CookieJar<'_>, callback: F) -> Result<R, ()>
where
	F: FnOnce(&'a str, &'a str) -> Fut,
	Fut: std::future::Future<Output = R>,
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

fn load_mmr_store() -> MurmurStore {
	// TODO: load from DB
	let mmr_store_file = File::open("mmr_store").expect("Unable to open file");
	let data: MurmurStore = serde_cbor::from_reader(mmr_store_file).unwrap();

	data
}

#[launch]
fn rocket() -> _ {
	rocket::build().mount("/", routes![login, create, prepare_execute])
}
