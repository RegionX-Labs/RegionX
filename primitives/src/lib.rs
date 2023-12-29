// This file is part of RegionX.
//
// RegionX is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// RegionX is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with RegionX.  If not, see <https://www.gnu.org/licenses/>.

#![cfg_attr(not(feature = "std"), no_std)]

use ink::prelude::vec::Vec;

pub mod coretime;
pub mod macros;
pub mod uniques;

/// Balance of an account.
pub type Balance = u64;

/// The type used for versioning metadata.
pub type Version = u32;

#[derive(scale::Encode, scale::Decode)]
pub enum RuntimeCall {
	#[codec(index = 37)]
	Uniques(uniques::UniquesCall),
}

/// A multi-format address wrapper for on-chain accounts.
#[derive(scale::Encode, scale::Decode, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "std", derive(Hash))]
pub enum MultiAddress<AccountId, AccountIndex> {
	/// It's an account ID (pubkey).
	Id(AccountId),
	/// It's an account index.
	Index(#[codec(compact)] AccountIndex),
	/// It's some arbitrary raw bytes.
	Raw(Vec<u8>),
	/// It's a 32 byte representation.
	Address32([u8; 32]),
	/// Its a 20 byte representation.
	Address20([u8; 20]),
}
