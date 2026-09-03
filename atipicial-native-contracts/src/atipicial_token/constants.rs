//! AtipicialCoin protocol constants and event names.
//!
//! Centralizes storage prefixes, default governance values, reward ratios, and
//! native event names so the contract root stays focused on the native-contract
//! surface.

/// C# `AtipicialCoin.Prefix_RegisterPrice`.
pub(in crate::atipicial_coin) const PREFIX_REGISTER_PRICE: u8 = 13;
/// C# default candidate register price: 1000 GAS, in datoshi (1000 * 1e8).
pub(in crate::atipicial_coin) const DEFAULT_REGISTER_PRICE: i64 = 1000 * 100_000_000;
/// C# `AtipicialCoin.Prefix_AtipicialDollarPerBlock`.
pub(in crate::atipicial_coin) const PREFIX_GAS_PER_BLOCK: u8 = 29;
/// C# default GAS-per-block at index 0: 5 GAS, in datoshi (5 * 1e8).
pub(in crate::atipicial_coin) const DEFAULT_GAS_PER_BLOCK: i64 = 5 * 100_000_000;
/// C# `AtipicialCoin.Prefix_Committee` - the cached `(pubkey, votes)` committee list.
pub(in crate::atipicial_coin) const PREFIX_COMMITTEE: u8 = 14;
/// C# `AtipicialCoin.Prefix_Candidate` - per-candidate `(Registered, Votes)` state.
pub(in crate::atipicial_coin) const PREFIX_CANDIDATE: u8 = 33;
/// C# `AtipicialCoin.Prefix_VoterRewardPerCommittee` - accumulated GAS-per-vote.
pub(in crate::atipicial_coin) const PREFIX_VOTER_REWARD_PER_COMMITTEE: u8 = 23;
/// C# `AtipicialCoin.Prefix_VotersCount` - total ATC that has voted (a BigInteger).
pub(in crate::atipicial_coin) const PREFIX_VOTERS_COUNT: u8 = 1;
/// C# `AtipicialCoin.AtipicialHolderRewardRatio` (10%).
pub(in crate::atipicial_coin) const ATC_HOLDER_REWARD_RATIO: i64 = 10;
/// C# `AtipicialCoin.CommitteeRewardRatio` (10%): the per-block GAS share minted to
/// the committee member selected by `index % committeeCount`.
pub(in crate::atipicial_coin) const COMMITTEE_REWARD_RATIO: i64 = 10;
/// C# `AtipicialCoin.VoterRewardRatio` (80%): the GAS share accrued (on committee
/// refresh blocks) to the voters of the committee.
pub(in crate::atipicial_coin) const VOTER_REWARD_RATIO: i64 = 80;
/// C# `AtipicialCoin.VoteFactor` (1e8): the zoom factor for per-vote GAS rewards.
pub(in crate::atipicial_coin) const VOTE_FACTOR: i64 = 100_000_000;
/// C# `AtipicialCoin.TotalAmount` = 1,000,000,000 ATC (decimals 0, so Factor = 1).
/// Atipicial Chain genesis supply: fixed cap, fully minted to the genesis
/// committee/validator set at `AtipicialCoin::initialize`.
pub(in crate::atipicial_coin) const ATC_TOTAL_AMOUNT: i64 = 1_000_000_000;

pub(crate) const ATC_CANDIDATE_STATE_CHANGED_EVENT: &str = "CandidateStateChanged";
pub(crate) const ATC_VOTE_EVENT: &str = "Vote";
pub(crate) const ATC_COMMITTEE_CHANGED_EVENT: &str = "CommitteeChanged";
