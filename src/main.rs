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
use rocket::http::Status;
use rocket::http::{Cookie, CookieJar};
use rocket::serde::{json::Json, Deserialize};
use sp_core::crypto::Ss58Codec;
use subxt::utils::{AccountId32, MultiAddress};
use subxt_signer::sr25519::dev;
use utils::{check_cookie, derive_seed};

const SALT: &str = "your-server-side-secret-salt";
const EPHEM_MSK: [u8; 32] = [1; 32];

#[derive(Deserialize)]
struct LoginRequest {
	username: String,
	password: String,
}
#[derive(Serialize, Deserialize)]
struct MMR {
	test: String,
	test2: String
}

// #[get("/insert")]
// async fn insert(db: Connection<Db>) {
// 	// db.database("admin").run_command(doc! {"ping": 1}, None).await;
// 	let test = String::from("abc");
// 	let test2 = String::from("cde");
// 	let doc = MMR{test, test2};
// 	let insert_result = db.database("Mmr").collection("mmrs").insert_one(doc, None).await;

// 	match insert_result {
// 		Err(e) => println!("Error inserting record : {e:?}"),
// 		Ok(insert) => {
// 			println!("succesfully inserted record, {insert:?}");
// 		}
// 	}
// 	// println!("Pinged your deployment. You successfully connected to MongoDB!");
// 	// Db.database("admin")
// }

// #[get("/delete")]
// async fn delete(db: Connection<Db>) {

// 	let test = String::from("abc");
// 	let test2 = String::from("cde");
// 	let object = MMR{test, test2};

// 	let bson_try = bson::to_bson(&object);
// 	match bson_try {
// 		Err(e) => println!("Error turning object into bson {e:?}"),
// 		Ok(bson_object) => {

// 			let collection: Collection<MMR> = db.database("Mmr").collection("mmrs");

// 			// Can delete by reconstructing the object or by using the object ID that is 
// 			// created on record insertion
// 			let query = bson_object.as_document().unwrap();
// 			// let object_id = ObjectId::parse_str("66fc1dc432627ab776148773").unwrap();
// 			// let query = doc!{"_id": object_id};
			

// 			let delete_result = collection.delete_one(query.clone(), None).await;
		
// 			// let delete_result = collection.delete_one(query.clone(), None).await;
// 			match delete_result {
// 					Err(e) => println!("Deletion error occurred: {e:?}"),
// 					Ok(success) => println!("Deletion Succeeded {success:?}")
// 				}
// 		}
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
	let seed = derive_seed(username, password, SALT);

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
			username.into(),
			seed.into(),
			EPHEM_MSK, // TODO: replace with an hkdf? https://github.com/ideal-lab5/murmur/issues/13
			schedule,
			round_pubkey_bytes,
		)
		.map_err(|_| Status::InternalServerError)?;
		// 3. add to storage
		store::write(mmr_store.clone());
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

		let store = store::load();
		let target_block_number = current_block_number + 1;

		let tx = murmur::prepare_execute(
			username.into(),
			seed.into(),
			target_block_number,
			store,
			balance_transfer_call,
		)
		.map_err(|_| Status::InternalServerError)?;

		// submit the tx using alice to sign it
		let _ = client.tx().sign_and_submit_then_watch_default(&tx, &dev::alice()).await;

		Ok("Transaction executed".to_string())
	})
	.await
}

#[launch]
fn rocket() -> _ {
	rocket::build().mount("/", routes![login, new, execute])
}
