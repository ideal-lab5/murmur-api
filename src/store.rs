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
use rocket::{futures::TryStreamExt, serde::Serialize};
use rocket_db_pools::{
	mongodb::{
		bson::{doc, oid::ObjectId},
		Client, Collection,
	},
	Database,
};
use std::borrow::Borrow;

// TODO: these should be moved to env vars like https://github.com/ideal-lab5/murmur-api/blob/8b7edd4f17ccdcd7a8c832525fce1df4781403fe/src/utils.rs#L67-L84
const DB_NAME: &str = "MurmurDB";
const COLLECTION_NAME: &str = "mmrs";
const DB_URI: &str = "mongodb+srv://murmurapi:GuVsTAEbQtNnnbPj@useast.m8j6h.mongodb.net/?retryWrites=true&w=majority&appName=USEast";

pub(crate) struct Store {
	pub(crate) col: Collection<MurmurStore>,
}

impl Store {
	pub(crate) async fn init() -> Self {
		let client = Client::with_uri_str(DB_URI).await.unwrap();
		let col = client.database(&DB_NAME).collection(&COLLECTION_NAME);
		Store { col }
	}

	pub(crate) async fn load(&self, object_id_string: String) -> MurmurStore {
		let object_id = ObjectId::parse_str(object_id_string).unwrap();

		let filter = doc! {"_id": object_id};

		let mut cursor = self.col.find(filter, None).await.unwrap();

		let mmr = cursor.try_next().await.unwrap().unwrap();

		mmr
	}

	pub(crate) async fn write<T>(&self, doc: T) -> String
	where
		T: Serialize + Borrow<MurmurStore>,
	{
		let insert_result = self.col.insert_one(doc, None).await;
		let mut object_id = String::new();

		match insert_result {
			Err(e) => println!("Error inserting record : {e:?}"),
			Ok(insert) => {
				println!("succesfully inserted record, {insert:?}");
				object_id = String::from(insert.inserted_id.as_str().unwrap());
			},
		}

		object_id
	}
}
