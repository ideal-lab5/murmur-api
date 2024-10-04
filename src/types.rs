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

use murmur::{BlockNumber, MurmurStore};
use rocket::http::Status;
use rocket::response::Responder;
use rocket::serde::{Deserialize, Serialize};
use rocket::{Request, Response};
use std::io::Cursor;

#[derive(Deserialize)]
pub(crate) struct AuthRequest {
	pub(crate) username: String,
	pub(crate) password: String,
}

#[derive(Deserialize)]
pub(crate) struct ExecuteRequest {
	pub(crate) amount: String,
	pub(crate) to: String,
	pub(crate) current_block: BlockNumber,
}

#[derive(Deserialize)]
pub(crate) struct CreateRequest {
	pub(crate) validity: u32,
	pub(crate) current_block: BlockNumber,
	pub(crate) round_pubkey: String,
}

#[derive(Serialize)]
pub(crate) struct Payload<CallData> {
	pallet_name: String,
	call_name: String,
	call_data: CallData,
}

// Implement the From trait for Payload to convert from TxPayload
impl<C, D> From<murmur::TxPayload<C>> for Payload<D>
where
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
pub(crate) struct Create {
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
pub(crate) struct CreateResponse {
	pub(crate) payload: Payload<Create>,
	pub(crate) store: MurmurStore,
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
pub(crate) struct Proxy {
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
pub(crate) struct ProxyResponse {
	pub(crate) payload: Payload<Proxy>,
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
