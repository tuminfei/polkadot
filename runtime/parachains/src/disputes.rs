// Copyright 2021 Parity Technologies (UK) Ltd.
// This file is part of Polkadot.

// Polkadot is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Polkadot is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Polkadot.  If not, see <http://www.gnu.org/licenses/>.

//! Runtime component for handling disputes of parachain candidates.

use sp_std::prelude::*;
use sp_std::result;
#[cfg(feature = "std")]
use sp_std::marker::PhantomData;
use primitives::v1::{
	Id as ParaId, ValidationCode, HeadData, SessionIndex, Hash, BlockNumber, CandidateHash,
	DisputeState,
};
use sp_runtime::{traits::One, DispatchResult, SaturatedConversion};
use frame_system::ensure_root;
use frame_support::{
	decl_storage, decl_module, decl_error, decl_event, ensure,
	traits::Get,
	weights::Weight,
};
use parity_scale_codec::{Encode, Decode};
use crate::{configuration, shared, initializer::SessionChangeNotification};
use sp_core::RuntimeDebug;

#[cfg(feature = "std")]
use serde::{Serialize, Deserialize};

pub trait Config:
	frame_system::Config +
	configuration::Config
{
	type Event: From<Event> + Into<<Self as frame_system::Config>::Event>;
}

decl_storage! {
	trait Store for Module<T: Config> as ParaDisputes {
		/// The last pruneed session, if any. All data stored by this module
		/// references sessions.
		LastPrunedSession: Option<SessionIndex>;
		// All ongoing or concluded disputes for the last several sessions.
		Disputes: double_map
			hasher(twox_64_concat) SessionIndex,
			hasher(blake2_128_concat) CandidateHash
			=> Option<DisputeState<T::BlockNumber>>;
		// All included blocks on the chain, as well as the block number in this chain that
		// should be reverted back to if the candidate is disputed and determined to be invalid.
		Included: double_map
			hasher(twox_64_concat) SessionIndex,
			hasher(blake2_128_concat) CandidateHash
			=> Option<T::BlockNumber>;
		// Maps session indices to a vector indicating the number of potentially-spam disputes
		// each validator is participating in. Potentially-spam disputes are remote disputes which have
		// fewer than `byzantine_threshold + 1` validators.
		//
		// The i'th entry of the vector corresponds to the i'th validator in the session.
		SpamSlots: map hasher(twox_64_concat) SessionIndex => Vec<u32>;
		// Whether the chain is frozen or not. Starts as `false`. When this is `true`,
		// the chain will not accept any new parachain blocks for backing or inclusion.
		// It can only be set back to `false` by governance intervention.
		Frozen: bool;
	}
}

decl_event! {
	pub enum Event {
		/// A dispute has been initiated. The boolean is true if the dispute is local. \[para_id\]
		DisputeInitiated(ParaId, CandidateHash, bool),
		/// A dispute has concluded for or against a candidate. The boolean is true if the candidate
		/// is deemed valid (for) and false if invalid (against).
		DisputeConcluded(ParaId, CandidateHash, bool),
		/// A dispute has timed out due to insufficient participation.
		DisputeTimedOut(ParaId, CandidateHash),
	}
}

decl_module! {
	/// The parachains configuration module.
	pub struct Module<T: Config> for enum Call where origin: <T as frame_system::Config>::Origin {
		fn deposit_event() = default;
	}
}

impl<T: Config> Module<T> {

}