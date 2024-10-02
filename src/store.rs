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
use rocket_db_pools::mongodb::{bson::doc, options::{FindOptions, InsertOneOptions}};
use std::fs::File;

use rocket_db_pools::Connection;
use rocket_db_pools::mongodb::Collection;
use rocket_db_pools::mongodb::bson::oid::ObjectId;
use rocket::{futures::TryStreamExt, serde::Serialize};

use crate::Db;

pub(crate) fn load_from_file() -> MurmurStore {
	// TODO: load from DB
	let mmr_store_file = File::open("mmr_store").expect("Unable to open file");
	let data: MurmurStore = serde_cbor::from_reader(mmr_store_file).unwrap();

	data
}

pub(crate) async fn load_from_db(object_id_string: String, db_name: &str, collection_name: &str, db:Connection<Db>, options: Option<FindOptions>) -> MurmurStore {

	let object_id = ObjectId::parse_str(object_id_string).unwrap();

	let filter = doc! {"_id": object_id};

	let mmr_collection: Collection<MurmurStore>= db.database(&db_name).collection(&collection_name);

	let mut cursor = mmr_collection.find(filter, options).await.unwrap();

	let mmr = cursor.try_next().await.unwrap().unwrap();

	mmr

}

/// Write the MMR data to a file
pub(crate) fn write_to_file(mmr_store: MurmurStore) {
	// TODO: write to DB
	let mmr_store_file = File::create("mmr_store").expect("It should create the file");
	serde_cbor::to_writer(mmr_store_file, &mmr_store).unwrap();
}

pub (crate) async fn write_to_db<T:Serialize>(db_name: &str, collection_name: &str, doc: T, db:Connection<Db>, options: Option<InsertOneOptions>) -> String {

	let insert_result = db.database(db_name).collection(collection_name).insert_one(doc, options).await;
	let mut object_id = String::new();

	match insert_result {
		Err(e) => println!("Error inserting record : {e:?}"),
		Ok(insert) => {
			println!("succesfully inserted record, {insert:?}");
			object_id = String::from(insert.inserted_id.as_str().unwrap());
		}
	}

	object_id
}
