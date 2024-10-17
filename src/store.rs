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

use hex::decode;
use murmur::MurmurStore;
use parity_scale_codec::{Decode, Encode};
use rocket::{
	futures::TryStreamExt,
	serde::{Deserialize, Serialize},
};
use rocket_db_pools::mongodb::{
	bson::doc, error::Error as DbError, options::UpdateOptions, results::UpdateResult, Client,
	Collection,
};
use std::{env, fmt::Display};

#[derive(Serialize, Deserialize)]
pub struct MurmurDbObject {
	pub mmr: String,
	pub username: String,
}

pub(crate) struct Store {
	pub(crate) col: Collection<MurmurDbObject>,
}

pub enum Error {
	/// Db error
	Db(DbError),
	Hex(hex::FromHexError),
	Codec(parity_scale_codec::Error),
}

impl Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Error::Db(e) => write!(f, "Db error: {}", e),
			Error::Hex(e) => write!(f, "Hex error: {}", e),
			Error::Codec(e) => write!(f, "Codec error: {}", e),
		}
	}
}

impl From<DbError> for Error {
	fn from(e: DbError) -> Self {
		Error::Db(e)
	}
}

impl From<hex::FromHexError> for Error {
	fn from(e: hex::FromHexError) -> Self {
		Error::Hex(e)
	}
}

impl From<parity_scale_codec::Error> for Error {
	fn from(e: parity_scale_codec::Error) -> Self {
		Error::Codec(e)
	}
}

impl Store {
	pub(crate) async fn init() -> Self {
		let client = Client::with_uri_str(env::var("DB_URI").unwrap()).await.unwrap();
		let col = client
			.database(&env::var("DB_NAME").unwrap())
			.collection(&env::var("DB_COLLECTION").unwrap());
		Store { col }
	}

	pub(crate) async fn load(&self, username: &str) -> Result<Option<MurmurStore>, Error> {
		let filter = doc! {"username": username};
		let mut mmr_cursor = self.col.find(filter, None).await?;

		let mmr_option = mmr_cursor.try_next().await?;
		match mmr_option {
			Some(mmr_db_object) => {
				let mmr_vec = decode(mmr_db_object.mmr)?;
				let mmr_store = MurmurStore::decode(&mut mmr_vec.as_slice())?;
				Ok(Some(mmr_store))
			},
			None => Ok(None),
		}
	}

	pub(crate) async fn write(
		&self,
		username: String,
		mmr: MurmurStore,
	) -> Result<UpdateResult, Error> {
		let mmr_encoded = hex::encode(MurmurStore::encode(&mmr));
		let murmur_data_object = MurmurDbObject { mmr: mmr_encoded, username: username.clone() };

		let filter = doc! { "username": &username };
		let update = doc! { "$set": { "mmr": &murmur_data_object.mmr, "username": &murmur_data_object.username } };
		let options = UpdateOptions::builder().upsert(true).build();

		let insert_result: rocket_db_pools::mongodb::results::UpdateResult =
			self.col.update_one(filter, update, options).await?;
		Ok(insert_result)
	}
}
