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

use murmur::MurmurStore;
use rocket::{
	futures::TryStreamExt,
	serde::{Deserialize, Serialize},
};
use rocket_db_pools::mongodb::{
	bson::doc, error::Error, results::InsertOneResult, Client, Collection,
};

// TODO move to env var https://github.com/ideal-lab5/murmur-api/issues/15
const DB_NAME: &str = "MurmurDB";
const DB_URI: &str = "mongodb+srv://murmurapi:GuVsTAEbQtNnnbPj@useast.m8j6h.mongodb.net/?retryWrites=true&w=majority&appName=USEast";
const COLLECTION_NAME: &str = "mmrs";

#[derive(Serialize, Deserialize)]
pub struct MurmurDbObject {
	pub mmr: MurmurStore,
	pub username: String,
}

pub(crate) struct Store {
	pub(crate) col: Collection<MurmurDbObject>,
}

impl Store {
	pub(crate) async fn init() -> Self {
		let client = Client::with_uri_str(DB_URI).await.unwrap();
		let col = client.database(&DB_NAME).collection(&COLLECTION_NAME);
		Store { col }
	}

	pub(crate) async fn load(&self, username: &str) -> Result<Option<MurmurStore>, Error> {
		let filter = doc! {"username": username};
		let mut mmr_cursor = self.col.find(filter, None).await?;

		let mmr_option = mmr_cursor.try_next().await?;
		match mmr_option {
			Some(mmr_db_object) => Ok(Some(mmr_db_object.mmr)),
			None => Ok(None),
		}
	}

	pub(crate) async fn write(
		&self,
		username: String,
		mmr: MurmurStore,
	) -> Result<InsertOneResult, Error> {
		let murmur_data_object = MurmurDbObject { mmr, username };
		let insert_result = self.col.insert_one(murmur_data_object, None).await;
		insert_result
	}
}
