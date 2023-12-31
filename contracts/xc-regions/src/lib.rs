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

#![cfg_attr(not(feature = "std"), no_std, no_main)]
#![feature(min_specialization)]

mod traits;
mod types;

// NOTE: This should be the collection ID of the underlying region collection.
const REGIONS_COLLECTION_ID: u32 = 42;

#[openbrush::implementation(PSP34)]
#[openbrush::contract(env = environment::ExtendedEnvironment)]
pub mod xc_regions {
	use crate::{
		traits::{regionmetadata_external, RegionMetadata},
		types::{VersionedRegion, XcRegionsError},
		REGIONS_COLLECTION_ID,
	};
	use ink::{
		codegen::{EmitEvent, Env},
		storage::Mapping,
	};
	use openbrush::traits::Storage;
	use primitives::{
		coretime::{RawRegionId, Region, RegionId},
		ensure,
		uniques::{ItemDetails, UniquesCall},
		RuntimeCall, Version,
	};
	use uniques_extension::UniquesExtension;

	#[ink(storage)]
	#[derive(Default, Storage)]
	pub struct XcRegions {
		#[storage_field]
		psp34: psp34::Data,
		/// A mapping that links RawRegionId to its corresponding region metadata.
		pub regions: Mapping<RawRegionId, Region>,
		/// A mapping that keeps track of the metadata version for each region.
		///
		/// This version gets incremented for a region each time it gets re-initialized.
		pub metadata_versions: Mapping<RawRegionId, Version>,
	}

	#[ink(event)]
	pub struct RegionInitialized {
		/// The identifier of the region that got initialized.
		#[ink(topic)]
		pub(crate) region_id: RawRegionId,
		/// The associated metadata.
		pub(crate) metadata: Region,
		/// The version of the metadata. This is incremented by the contract each time the same
		/// region is initialized.
		pub(crate) version: Version,
	}

	#[ink(event)]
	pub struct RegionRemoved {
		/// The identifier of the region that got removed.
		#[ink(topic)]
		pub(crate) region_id: RawRegionId,
	}

	#[overrider(PSP34)]
	fn collection_id(&self) -> Id {
		Id::U32(REGIONS_COLLECTION_ID)
	}

	impl RegionMetadata for XcRegions {
		/// A function for minting a wrapped xcRegion initializing the metadata of it. It can only
		/// be called if the specified region exists on this chain and the caller is the actual
		/// owner of the region.
		///
		/// ## Arguments:
		/// - `raw_region_id` - The `u128` encoded region identifier.
		/// - `region` - The corresponding region metadata.
		///
		/// This function conducts a sanity check to verify that the metadata derived from the
		/// `raw_region_id` aligns with the respective components of the metadata supplied through
		/// the region argument.
		///
		/// If this is not the first time that this region is inititalized, the metadata version
		/// will get incremented.
		///
		/// ## Events:
		/// On success this ink message emits the `RegionInitialized` event.
		#[ink(message)]
		fn init(
			&mut self,
			raw_region_id: RawRegionId,
			region: Region,
		) -> Result<(), XcRegionsError> {
			let caller = self.env().caller();
			ensure!(
				Some(caller) == self._uniques_owner(raw_region_id),
				XcRegionsError::CannotInitialize
			);

			// Cannot initialize a region that already has metadata stored.
			ensure!(self.regions.get(raw_region_id).is_none(), XcRegionsError::CannotInitialize);

			// Do a sanity check to ensure that the provided region metadata matches with the
			// metadata extracted from the region id.
			let region_id = RegionId::from(raw_region_id);
			ensure!(region_id.begin == region.begin, XcRegionsError::InvalidMetadata);
			ensure!(region_id.core == region.core, XcRegionsError::InvalidMetadata);
			ensure!(region_id.mask == region.mask, XcRegionsError::InvalidMetadata);

			// After passing all checks we will transfer the region to the contract and mint a
			// wrapped xcRegion token.
			let contract = self.env().account_id();
			self._transfer_approved(raw_region_id, contract)?;

			let new_version = if let Some(version) = self.metadata_versions.get(raw_region_id) {
				version.saturating_add(1)
			} else {
				Default::default()
			};

			self.metadata_versions.insert(raw_region_id, &new_version);
			self.regions.insert(raw_region_id, &region);

			psp34::InternalImpl::_mint_to(self, caller, Id::U128(raw_region_id))
				.map_err(|err| XcRegionsError::Psp34(err))?;

			self.env().emit_event(RegionInitialized {
				region_id: raw_region_id,
				metadata: region,
				version: new_version,
			});

			Ok(())
		}

		/// A function to retrieve all metadata associated with a specific region. This function
		/// verifies the region's existence on this chain prior to fetching its metadata.
		///
		/// The function returns a `VersionedRegion`, encompassing the version of the retrieved
		/// metadata that is intended for client-side verification.
		///
		/// ## Arguments:
		/// - `raw_region_id` - The `u128` encoded region identifier.
		#[ink(message)]
		fn get_metadata(&self, region_id: RawRegionId) -> Result<VersionedRegion, XcRegionsError> {
			// We must first ensure that the region is still present on this chain before retrieving
			// the metadata.
			ensure!(self._uniques_exists(region_id), XcRegionsError::RegionNotFound);

			let Some(region) = self.regions.get(region_id) else {
				return Err(XcRegionsError::MetadataNotFound)
			};
			let Some(version) = self.metadata_versions.get(region_id) else {
				// This should never really happen; if a region has its metadata stored, its version
				// should be stored as well.
				return Err(XcRegionsError::VersionNotFound)
			};

			Ok(VersionedRegion { version, region })
		}

		/// A function for removing the metadata associated with a region.
		///
		/// This function is callable by anyone, and the metadata is removed successfully if the
		/// specific region no longer exists on this chain.
		///
		/// ## Arguments:
		/// - `raw_region_id` - The `u128` encoded region identifier.
		///
		/// ## Events:
		/// On success this ink message emits the `RegionRemoved` event.
		#[ink(message)]
		fn remove(&mut self, region_id: RawRegionId) -> Result<(), XcRegionsError> {
			let id = Id::U128(region_id);
			let owner = psp34::PSP34Impl::owner_of(self, id.clone())
				.ok_or(XcRegionsError::RegionNotFound)?;

			self.regions.remove(region_id);
			psp34::InternalImpl::_transfer_token(self, owner, id, Default::default())
				.map_err(|err| XcRegionsError::Psp34(err))?;

			self.env().emit_event(RegionRemoved { region_id });
			Ok(())
		}
	}

	impl XcRegions {
		#[ink(constructor)]
		pub fn new() -> Self {
			Default::default()
		}
	}

	// Internal functions:
	impl XcRegions {
		fn _transfer_approved(
			&self,
			region_id: RawRegionId,
			dest: AccountId,
		) -> Result<(), XcRegionsError> {
			self.env()
				.call_runtime(&RuntimeCall::Uniques(UniquesCall::Transfer {
					collection: REGIONS_COLLECTION_ID,
					item: region_id,
					dest: dest.into(),
				}))
				.map_err(|_| XcRegionsError::RuntimeError)?;

			Ok(())
		}

		/// Returns whether the region exists on this chain or not.
		fn _uniques_exists(&self, region_id: RawRegionId) -> bool {
			self._uniques_item(region_id).is_some()
		}

		/// Returns the details of an item within a collection.
		fn _uniques_item(&self, item_id: RawRegionId) -> Option<ItemDetails> {
			self.env().extension().item(REGIONS_COLLECTION_ID, item_id).ok()?
		}

		/// The owner of the specific item.
		fn _uniques_owner(&self, region_id: RawRegionId) -> Option<AccountId> {
			self.env().extension().owner(REGIONS_COLLECTION_ID, region_id).ok()?
		}
	}

	#[cfg(all(test, feature = "e2e-tests"))]
	pub mod tests {
		use super::*;
		use crate::{
			traits::regionmetadata_external::RegionMetadata,
			types::{VersionedRegion, XcRegionsError},
			REGIONS_COLLECTION_ID,
		};
		use environment::ExtendedEnvironment;
		use ink::{
			env::{test::DefaultAccounts, DefaultEnvironment},
			primitives::AccountId,
		};
		use ink_e2e::{subxt::dynamic::Value, AccountKeyring::Alice, MessageBuilder};
		use openbrush::contracts::psp34::psp34_external::PSP34;
		use primitives::{address_of, assert_ok};

		type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

		#[ink_e2e::test(environment = ExtendedEnvironment)]
		async fn init_non_existing_region_fails(
			mut client: ink_e2e::Client<C, E>,
		) -> E2EResult<()> {
			let constructor = XcRegionsRef::new();
			let contract_acc_id = client
				.instantiate("xc-regions", &ink_e2e::alice(), constructor, 0, None)
				.await
				.expect("instantiate failed")
				.account_id;

			let raw_region_id = 0u128;
			let region = Region::default();

			let init = MessageBuilder::<ExtendedEnvironment, XcRegionsRef>::from_account_id(
				contract_acc_id.clone(),
			)
			.call(|xc_regions| xc_regions.init(raw_region_id, region.clone()));
			let init_result = client.call(&ink_e2e::alice(), init, 0, None).await;
			assert!(init_result.is_err(), "Init for non existing region should fail");

			Ok(())
		}

		#[ink_e2e::test(environment = ExtendedEnvironment)]
		async fn init_works(mut client: E2EBackend) -> E2EResult<()> {
			let constructor = XcRegionsRef::new();
			let contract_acc_id = client
				.instantiate("xc-regions", &ink_e2e::alice(), constructor, 0, None)
				.await
				.expect("instantiate failed")
				.account_id;

			let raw_region_id = 0u128;
			let region = Region::default();

			// Create region: collection
			let call_data = vec![
				Value::u128(REGIONS_COLLECTION_ID.into()),
				Value::unnamed_variant("Id", [Value::from_bytes(&address_of!(Alice))]),
			];
			client
				.runtime_call(&ink_e2e::alice(), "Uniques", "create", call_data)
				.await
				.expect("creating a collection failed");

			// Mint region:
			let call_data = vec![
				Value::u128(REGIONS_COLLECTION_ID.into()),
				Value::u128(raw_region_id.into()),
				Value::unnamed_variant("Id", [Value::from_bytes(&address_of!(Alice))]),
			];
			client
				.runtime_call(&ink_e2e::alice(), "Uniques", "mint", call_data)
				.await
				.expect("minting a region failed");

			// Approve transfer region:
			let call_data = vec![
				Value::u128(REGIONS_COLLECTION_ID.into()),
				Value::u128(raw_region_id.into()),
				Value::unnamed_variant("Id", [Value::from_bytes(contract_acc_id)]),
			];
			client
				.runtime_call(&ink_e2e::alice(), "Uniques", "approve_transfer", call_data)
				.await
				.expect("approving transfer failed");

			let init = MessageBuilder::<ExtendedEnvironment, XcRegionsRef>::from_account_id(
				contract_acc_id.clone(),
			)
			.call(|xc_regions| xc_regions.init(raw_region_id, region.clone()));
			let init_result = client.call(&ink_e2e::alice(), init, 0, None).await;
			assert!(init_result.is_ok(), "Init should work");

			let balance_of = MessageBuilder::<ExtendedEnvironment, XcRegionsRef>::from_account_id(
				contract_acc_id.clone(),
			)
			.call(|xc_regions| xc_regions.balance_of(address_of!(Alice)));
			let balance_of_res = client.call_dry_run(&ink_e2e::alice(), &balance_of, 0, None).await;

			assert_eq!(balance_of_res.return_value(), 1);

			let get_metadata =
				MessageBuilder::<ExtendedEnvironment, XcRegionsRef>::from_account_id(
					contract_acc_id.clone(),
				)
				.call(|xc_regions| xc_regions.get_metadata(raw_region_id));
			let get_metadata_res =
				client.call_dry_run(&ink_e2e::alice(), &get_metadata, 0, None).await;

			assert_eq!(get_metadata_res.return_value(), Ok(VersionedRegion { version: 0, region }));

			Ok(())
		}

		#[ink_e2e::test(environment = ExtendedEnvironment)]
		async fn init_fails_with_incorrect_region_id(mut client: E2EBackend) -> E2EResult<()> {
			let constructor = XcRegionsRef::new();
			let contract_acc_id = client
				.instantiate("xc-regions", &ink_e2e::alice(), constructor, 0, None)
				.await
				.expect("instantiate failed")
				.account_id;

			let raw_region_id = 0u128;
			let mut region = Region::default();

			// Create region: collection
			let call_data = vec![
				Value::u128(REGIONS_COLLECTION_ID.into()),
				Value::unnamed_variant("Id", [Value::from_bytes(&address_of!(Alice))]),
			];
			client
				.runtime_call(&ink_e2e::alice(), "Uniques", "create", call_data)
				.await
				.expect("creating a collection failed");

			// Mint region:
			let call_data = vec![
				Value::u128(REGIONS_COLLECTION_ID.into()),
				Value::u128(raw_region_id.into()),
				Value::unnamed_variant("Id", [Value::from_bytes(&address_of!(Alice))]),
			];
			client
				.runtime_call(&ink_e2e::alice(), "Uniques", "mint", call_data)
				.await
				.expect("minting a region failed");

			// Approve transfer region:
			let call_data = vec![
				Value::u128(REGIONS_COLLECTION_ID.into()),
				Value::u128(raw_region_id.into()),
				Value::unnamed_variant("Id", [Value::from_bytes(contract_acc_id)]),
			];
			client
				.runtime_call(&ink_e2e::alice(), "Uniques", "approve_transfer", call_data)
				.await
				.expect("approving transfer failed");

			// Corrupt the metadata. The contract will notice since begin is part of the `RegionId`.
			region.begin = 42;
			let init = MessageBuilder::<ExtendedEnvironment, XcRegionsRef>::from_account_id(
				contract_acc_id.clone(),
			)
			.call(|xc_regions| xc_regions.init(raw_region_id, region.clone()));
			let init_result = client.call(&ink_e2e::alice(), init, 0, None).await;
			assert!(init_result.is_err(), "Init with incorrect region.begin should fail");

			// Works after resetting the default:
			region.begin = Default::default();
			let init = MessageBuilder::<ExtendedEnvironment, XcRegionsRef>::from_account_id(
				contract_acc_id.clone(),
			)
			.call(|xc_regions| xc_regions.init(raw_region_id, region.clone()));
			let init_result = client.call(&ink_e2e::alice(), init, 0, None).await;
			assert!(init_result.is_ok(), "Init with correct region.begin should succeed");

			Ok(())
		}
	}
}
