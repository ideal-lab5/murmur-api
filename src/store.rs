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
use rocket_db_pools::mongodb::error::Error;
use rocket_db_pools::mongodb::bson::doc;
use rocket::serde::{Deserialize, Serialize};
use rocket_db_pools::mongodb::results::InsertOneResult;
use rocket_db_pools::{
	mongodb::Collection,
	Connection,
};

use crate::Db;
use murmur::MurmurStore;
const DB_NAME: &str = "MurmurDB";
const COLLECTION_NAME: &str = "mmrs";

#[derive (Serialize, Deserialize)]
pub struct MurmurDbObject {
	pub mmr: MurmurStore,
	pub username: String
}

pub(crate) async fn load(
	username: &str,
	db: Connection<Db>,
	options: Option<FindOptions>,
) -> Result<MurmurDbObject, Error> {

	let filter = doc! {"username": username};
	let mmr_collection: Collection<MurmurDbObject> = db.database(&DB_NAME).collection(&COLLECTION_NAME);
	let cursor_result = mmr_collection.find(filter, options).await;

	match cursor_result {
		Err(e) => Err(e),
		Ok(mut mmr_cursor) => {
			let mmr_next = mmr_cursor.try_next().await;
			match mmr_next {
				Err(e) => Err(e),
				Ok(mmr_option) => {
					let mmr = mmr_option.unwrap();
					Ok(mmr)
				}
			}
		}
	}
}

pub(crate) async fn write(
	username: String,
	mmr: MurmurStore,
	db: Connection<Db>,
	options: Option<InsertOneOptions>,
) -> Result<InsertOneResult, Error>{
	let murmur_data_object = MurmurDbObject{mmr, username};
	let mmr_collection: Collection<MurmurDbObject> = db.database(DB_NAME).collection(COLLECTION_NAME);
	let insert_result = mmr_collection.insert_one(murmur_data_object, options).await;
	insert_result
}