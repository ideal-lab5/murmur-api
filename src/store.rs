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

use rocket::futures::TryStreamExt;
use rocket_db_pools::mongodb::options::{FindOptions, InsertOneOptions};
use rocket_db_pools::mongodb::Client;
use rocket_db_pools::mongodb::error::Error;
use rocket_db_pools::mongodb::bson::doc;
use rocket_db_pools::mongodb::results::InsertOneResult;
use rocket_db_pools::mongodb::Collection;
use rocket::serde::{Deserialize, Serialize};


use murmur::MurmurStore;
const DB_NAME: &str = "MurmurDB";
const DB_URI: &str = "mongodb+srv://murmurapi:GuVsTAEbQtNnnbPj@useast.m8j6h.mongodb.net/?retryWrites=true&w=majority&appName=USEast";
const COLLECTION_NAME: &str = "mmrs";

#[derive (Serialize, Deserialize)]
pub struct MurmurDbObject {
	pub mmr: MurmurStore,
	pub username: String
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

	pub(crate) async fn load(
		&self,
		username: &str,
		options: Option<FindOptions>,
	) -> Result<Option<MurmurStore>, Error> {
	
		let filter = doc! {"username": username};
		// let mmr_collection: Collection<MurmurDbObject> = db.database(&DB_NAME).collection(&COLLECTION_NAME);
		let cursor_result = self.col.find(filter, options).await;
	
		match cursor_result {
			Err(e) => Err(e),
			Ok(mut mmr_cursor) => {
				let mmr_next = mmr_cursor.try_next().await;
				match mmr_next {
					Err(e) => Err(e),
					Ok(mmr_option) => {
						match mmr_option {
							Some(mmr_db_object) => {
								Ok(Some(mmr_db_object.mmr))
							},
							None => Ok(None)
						}
					}
				}
			}
		}
	}
	
	pub(crate) async fn write(
		&self,
		username: String,
		mmr: MurmurStore,
		options: Option<InsertOneOptions>,
	) -> Result<InsertOneResult, Error>{
		let murmur_data_object = MurmurDbObject{mmr, username};
		let insert_result = self.col.insert_one(murmur_data_object, options).await;
		insert_result
	}
}
