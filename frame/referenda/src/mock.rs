// This file is part of Substrate.

// Copyright (C) 2017-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! The crate's tests.

use super::*;
use crate as pallet_referenda;
use codec::{Encode, Decode};
use frame_support::{
	assert_ok, ord_parameter_types, parameter_types,
	traits::{Contains, EqualPrivilegeOnly, GenesisBuild, OnInitialize, SortedMembers, OriginTrait},
	weights::Weight,
};
use frame_system::{EnsureRoot, EnsureSignedBy};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	Perbill,
};

const MAX_PROPOSALS: u32 = 100;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Scheduler: pallet_scheduler::{Pallet, Call, Storage, Config, Event<T>},
		Referenda: pallet_referenda::{Pallet, Call, Storage, Config<T>, Event<T>},
	}
);

// Test that a fitlered call can be dispatched.
pub struct BaseFilter;
impl Contains<Call> for BaseFilter {
	fn contains(call: &Call) -> bool {
		!matches!(call, &Call::Balances(pallet_balances::Call::set_balance { .. }))
	}
}

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub BlockWeights: frame_system::limits::BlockWeights =
		frame_system::limits::BlockWeights::simple_max(1_000_000);
}
impl frame_system::Config for Test {
	type BaseCallFilter = BaseFilter;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Index = u64;
	type BlockNumber = u64;
	type Call = Call;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
}
parameter_types! {
	pub MaximumSchedulerWeight: Weight = Perbill::from_percent(80) * BlockWeights::get().max_block;
}
impl pallet_scheduler::Config for Test {
	type Event = Event;
	type Origin = Origin;
	type PalletsOrigin = OriginCaller;
	type Call = Call;
	type MaximumWeight = MaximumSchedulerWeight;
	type ScheduleOrigin = EnsureRoot<u64>;
	type MaxScheduledPerBlock = ();
	type WeightInfo = ();
	type OriginPrivilegeCmp = EqualPrivilegeOnly;
}
parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
	pub const MaxLocks: u32 = 10;
}
impl pallet_balances::Config for Test {
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type MaxLocks = MaxLocks;
	type Balance = u64;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
}
parameter_types! {
	pub const LaunchPeriod: u64 = 2;
	pub const VotingPeriod: u64 = 2;
	pub const FastTrackVotingPeriod: u64 = 2;
	pub const MinimumDeposit: u64 = 1;
	pub const AlarmInterval: u64 = 2;
	pub const VoteLockingPeriod: u64 = 3;
	pub const CooloffPeriod: u64 = 2;
	pub const MaxVotes: u32 = 100;
	pub const MaxProposals: u32 = MAX_PROPOSALS;
	pub static PreimageByteDeposit: u64 = 0;
	pub static InstantAllowed: bool = false;
	pub const SubmissionDeposit: u64 = 2;
	pub const MaxQueued: u32 = 2;
	pub const UndecidingTimeout: u64 = 10;
}
ord_parameter_types! {
	pub const One: u64 = 1;
	pub const Two: u64 = 2;
	pub const Three: u64 = 3;
	pub const Four: u64 = 4;
	pub const Five: u64 = 5;
	pub const Six: u64 = 6;
}
pub struct OneToFive;
impl SortedMembers<u64> for OneToFive {
	fn sorted_members() -> Vec<u64> {
		vec![1, 2, 3, 4, 5]
	}
	#[cfg(feature = "runtime-benchmarks")]
	fn add(_m: &u64) {}
}

pub struct TestTracksInfo;
impl TracksInfo<u64, u64> for TestTracksInfo {
	type Id = u8;
	type Origin = <Origin as OriginTrait>::PalletsOrigin;
	fn tracks() -> &'static [(Self::Id, TrackInfo<u64, u64>)] {
		static DATA: [(u8, TrackInfo<u64, u64>); 2] = [
			(0u8, TrackInfo {
				name: "root",
				max_deciding: 1,
				decision_deposit: 10,
				prepare_period: 4,
				decision_period: 4,
				confirm_period: 4,
				min_enactment_period: 4,
				min_approval: Curve::LinearDecreasing {
					begin: Perbill::from_percent(100),
					delta: Perbill::from_percent(50),
				},
				min_turnout: Curve::LinearDecreasing {
					begin: Perbill::from_percent(100),
					delta: Perbill::from_percent(100),
				},
			}),
			(1u8, TrackInfo {
				name: "none",
				max_deciding: 3,
				decision_deposit: 1,
				prepare_period: 2,
				decision_period: 2,
				confirm_period: 2,
				min_enactment_period: 2,
				min_approval: Curve::LinearDecreasing {
					begin: Perbill::from_percent(55),
					delta: Perbill::from_percent(5),
				},
				min_turnout: Curve::LinearDecreasing {
					begin: Perbill::from_percent(10),
					delta: Perbill::from_percent(10),
				},
			}),
		];
		&DATA[..]
	}
	fn track_for(id: &Self::Origin) -> Result<Self::Id, ()> {
		use sp_std::convert::TryFrom;
		if let Ok(system_origin) = frame_system::RawOrigin::try_from(id.clone()) {
			match system_origin {
				frame_system::RawOrigin::Root => Ok(0),
				frame_system::RawOrigin::None => Ok(1),
				_ => Err(()),
			}
		} else {
			Err(())
		}
	}
}

impl Config for Test {
	type WeightInfo = ();
	type Call = Call;
	type Event = Event;
	type Scheduler = Scheduler;
	type Currency = pallet_balances::Pallet<Self>;
	type CancelOrigin = EnsureSignedBy<Four, u64>;
	type KillOrigin = EnsureRoot<u64>;
	type Slash = ();
	type Votes = u32;
	type Tally = Tally;
	type SubmissionDeposit = SubmissionDeposit;
	type MaxQueued = MaxQueued;
	type UndecidingTimeout = UndecidingTimeout;
	type AlarmInterval = AlarmInterval;
	type Tracks = TestTracksInfo;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	pallet_balances::GenesisConfig::<Test> {
		balances: vec![(1, 10), (2, 20), (3, 30), (4, 40), (5, 50), (6, 60)],
	}
	.assimilate_storage(&mut t)
	.unwrap();
	pallet_referenda::GenesisConfig::<Test>::default()
		.assimilate_storage(&mut t)
		.unwrap();
	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}

/// Execute the function two times, with `true` and with `false`.
pub fn new_test_ext_execute_with_cond(execute: impl FnOnce(bool) -> () + Clone) {
	new_test_ext().execute_with(|| (execute.clone())(false));
	new_test_ext().execute_with(|| execute(true));
}

#[derive(Encode, Debug, Decode, TypeInfo, Eq, PartialEq, Clone, Default)]
pub struct Tally {
	pub ayes: u32,
	pub nays: u32,
}

impl VoteTally<u32> for Tally {
    fn ayes(&self) -> u32 {
        self.ayes
    }

    fn turnout(&self) -> Perbill {
        Perbill::from_percent(self.ayes + self.nays)
    }

    fn approval(&self) -> Perbill {
        Perbill::from_rational(self.ayes, self.ayes + self.nays)
    }
}

pub fn set_balance_proposal(value: u64) -> Vec<u8> {
	Call::Balances(pallet_balances::Call::set_balance { who: 42, new_free: value, new_reserved: 0 })
		.encode()
}

pub fn set_balance_proposal_hash(value: u64) -> H256 {
	use sp_core::Hasher;
	BlakeTwo256::hash(&set_balance_proposal(value)[..])
}

pub fn propose_set_balance(who: u64, value: u64, delay: u64) -> DispatchResult {
	Referenda::submit(
		Origin::signed(who),
		frame_system::RawOrigin::Root.into(),
		set_balance_proposal_hash(value),
		AtOrAfter::After(delay),
	)
}

pub fn next_block() {
	System::set_block_number(System::block_number() + 1);
	Scheduler::on_initialize(System::block_number());
}

pub fn fast_forward_to(n: u64) {
	while System::block_number() < n {
		next_block();
	}
}

pub fn begin_referendum() -> ReferendumIndex {
	System::set_block_number(0);
	assert_ok!(propose_set_balance(1, 2, 1));
	fast_forward_to(2);
	0
}

pub fn tally(r: ReferendumIndex) -> Tally {
	Referenda::ensure_ongoing(r).unwrap().tally
}
