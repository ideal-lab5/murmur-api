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
use std::fs::File;

pub(crate) fn load() -> MurmurStore {
	// TODO: load from DB
	let mmr_store_file = File::open("mmr_store").expect("Unable to open file");
	let data: MurmurStore = serde_cbor::from_reader(mmr_store_file).unwrap();

	data
}

/// Write the MMR data to a file
pub(crate) fn write(mmr_store: MurmurStore) {
	// TODO: write to DB
	let mmr_store_file = File::create("mmr_store").expect("It should create the file");
	serde_cbor::to_writer(mmr_store_file, &mmr_store).unwrap();
}
