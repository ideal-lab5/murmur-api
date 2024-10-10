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

use murmur::{BlockNumber, CreateData, ProxyData};
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
	pub(crate) runtime_call: Vec<u8>,
	pub(crate) current_block: BlockNumber,
}

#[derive(Deserialize)]
pub(crate) struct CreateRequest {
	pub(crate) validity: u32,
	pub(crate) current_block: BlockNumber,
	pub(crate) round_pubkey: String,
}

#[derive(Serialize)]
pub(crate) struct CreateResponse {
	pub(crate) username: Vec<u8>,
	pub(crate) create_data: CreateData,
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
pub(crate) struct ExecuteResponse {
	pub(crate) username: Vec<u8>,
	pub(crate) proxy_data: ProxyData,
}

impl<'r> Responder<'r, 'static> for ExecuteResponse {
	fn respond_to(self, _: &'r Request<'_>) -> rocket::response::Result<'static> {
		let json_response =
			serde_json::to_string(&self).map_err(|_| Status::InternalServerError)?;
		Response::build()
			.header(rocket::http::ContentType::JSON)
			.sized_body(json_response.len(), Cursor::new(json_response))
			.ok()
	}
}
