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

pub(crate) fn pubkey_to_bytes(pubkey: &str) -> Result<Vec<u8>, String> {
	let pubkey = if let Some(stripped) = pubkey.strip_prefix("0x") { stripped } else { pubkey };

	let round_pubkey_bytes =
		hex::decode(pubkey).map_err(|_| format!("Wrong input `{:?}`", pubkey))?;
	Ok(round_pubkey_bytes)
}

/// Convert a string of comma-separated integers to a fixed-size array of bytes
pub(crate) fn str_to_bytes(input: &str) -> Result<[u8; 32], &str> {
	let vec: Vec<u8> = input
		.split(',')
		.map(|s| {
			s.trim()
				.parse()
				.unwrap_or_else(|_| panic!("Invalid integer in input {}", input))
		})
		.collect();
	let sized: [u8; 32] = vec.try_into().map_err(|_| "Vector length is not 32")?;
	Ok(sized)
}
