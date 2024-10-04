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

use sp_core::crypto::Ss58Codec;
use subxt::utils::AccountId32;

pub(crate) fn pubkey_to_bytes(pubkey: &str) -> Result<Vec<u8>, String> {
	let pubkey = if pubkey.starts_with("0x") { &pubkey[2..] } else { pubkey };

	let round_pubkey_bytes =
		hex::decode(pubkey).map_err(|_| format!("Wrong input `{:?}`", pubkey))?;
	Ok(round_pubkey_bytes)
}

pub(crate) fn ss58_to_account_id(ss58: &str) -> Result<AccountId32, String> {
	let from_ss58 = sp_core::crypto::AccountId32::from_ss58check(ss58)
		.map_err(|_| format!("Wrong input `{:?}`", ss58))?;
	let bytes: &[u8] = from_ss58.as_ref();
	let from_ss58_sized: [u8; 32] =
		bytes.try_into().map_err(|_| format!("Wrong input `{:?}`", ss58))?;
	Ok(AccountId32::from(from_ss58_sized))
}

pub(crate) fn str_to_int<I>(input: &str) -> Result<I, String>
where
	I: std::str::FromStr,
{
	let res = input.parse::<I>().map_err(|_| format!("Wrong input `{:?}`", input))?;
	Ok(res)
}
