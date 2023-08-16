//! # Double Auction
//!
//! ## Overview
//!
//! This module provides a basic implement for order-book style on-chain double auctioning.
//!
//! This is the matching layer of a decentralized marketplace for electrical energy.
//! Sellers are categorized based on how much electricity they intend to sell.
//! Buyers are also categorized based on how much electricity they intend to buy.
//!
//! The highest bidding buyer in the same category with a seller is matched
//!  when the auction period of a seller is over.
//!
//! The seller has the benefit of getting the best price at a given point in time for their category,
//! while the buyer can choose a margin of safety for every buy.
//!
//! Auctions are executed in the auction execution queue based on their ending time
//!
//! NOTE: this mocdule does not implement how payment is handled.
//!
//! `Data`:     
//!     --  AuctionData<AccountId, BlockNumber, Bid, Tier> {
//!             pub auction_id: u64,
//!             pub seller_id: AccountId,
//!             pub quantity: u128,
//!             pub starting_bid: Bid,
//!             pub bids: Vec<Bid>,
//!             pub auction_period: BlockNumber,
//!             pub auction_status: AuctionStatus,
//!             pub start_at: BlockNumber,
//!             pub end_at: BlockNumber,
//!             pub highest_bid: Bid,
//!             pub auction_category: Tier,
//!         }
//!     -- AuctionInfo<AccountId, BlockNumber, Bid, Tier, PartyType> {
//!             pub participant_id: Option<AccountId>,
//!             pub party_type: PartyType,
//!             pub auctions: Vec<AuctionData<AccountId, BlockNumber, Bid, Tier>>, // Maximum length of 5
//!         }
//!     -- Tier: u128,  // 0, 1, 2, ...
//!     -- Auctions { auction_id -> AuctionData }
//!     -- AuctionsOf { account_id -> AuctionInfo }
//!
//! `Interface`:
//!     -- new(...)
//!     -- bid(...)
//!     -- cancel(...)
//!
//! `Hooks`:
//!     -- on_auction_ended
//!
//! `RPC`:
//!

#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
// pub mod weights;
// pub use weights::*;

#[frame_support::pallet]
pub mod pallet {

    use super::*;
    use frame_support::inherent::Vec;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        // /// Type representing the weight of this pallet
        // type WeightInfo: WeightInfo;
    }

    //////////////////////
    // Storage types   //
    /////////////////////

    // Buyers bid
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
    pub struct Bid<AccountId> {
        pub bidder: AccountId,
        pub bid: u128,
    }

    // Status of an auction, live auctions accepts bids
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
    pub enum AuctionStatus {
        Open,
        Closed,
    }
    impl Default for AuctionStatus {
        fn default() -> Self {
            AuctionStatus::Open
        }
    }

    // Essential data for an auction
    #[derive(Clone, Encode, Decode, Default, Eq, PartialEq, RuntimeDebug, TypeInfo)]
    pub struct AuctionData<AccountId, BlockNumber, Bid, Tier> {
        pub auction_id: u64,
        pub seller_id: AccountId,
        pub quantity: u128,
        pub starting_bid: Bid,
        pub bids: Vec<Bid>,
        pub auction_period: BlockNumber,
        pub auction_status: AuctionStatus,
        pub start_at: BlockNumber,
        pub end_at: BlockNumber,
        pub highest_bid: Bid,
        pub auction_category: Tier,
    }

    // Tier of an auction sale
    // Higher quantity of energy for sale leads to higher tier
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
    pub struct Tier {
        pub level: u32,
    }
    impl Default for Tier {
        fn default() -> Self {
            Tier { level: 1 }
        }
    }

    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
    pub enum PartyType {
        Seller,
        Buyer,
    }
    impl Default for PartyType {
        fn default() -> Self {
            PartyType::Seller
        }
    }

    // Auctions linked to an auction participant
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
    pub struct AuctionInfo<AccountId, PartyType> {
        pub participant_id: Option<AccountId>,
        pub party_type: PartyType,
        pub auctions: Vec<u64>, // Maximum of 5 auction id
    }

    impl<AccountId> Default for AuctionInfo<AccountId, PartyType> {
        fn default() -> Self {
            AuctionInfo {
                participant_id: None,
                party_type: PartyType::Seller,
                auctions: vec![],
            }
        }
    }

    //////////////////////
    // Storage item    //
    /////////////////////
    #[pallet::storage]
    #[pallet::getter(fn auctions_index)]
    pub(super) type AuctionIndex<T: Config> = StorageValue<_, u64>;

    /// Stores on-going and future auctions of participants
    /// Maximum of 5 auction cachesd at a time
    // TODO: use BoundedVec
    #[pallet::storage]
    #[pallet::getter(fn auctions_of)]
    pub(super) type AuctionsOf<T: Config> = StorageMap<
        _,
        Twox64Concat,
        T::AccountId,
        AuctionInfo<T::AccountId, PartyType>,
        OptionQuery,
    >;

    /// Stores on-going and future auctions of participants
    /// Closed auction are removed to optimize on-chain storage
    #[pallet::storage]
    #[pallet::getter(fn auctions)]
    pub(super) type Auctions<T: Config> = StorageMap<
        _,
        Twox64Concat,
        u64, // auction id
        AuctionData<T::AccountId, T::BlockNumber, Bid<T::AccountId>, Tier>,
        OptionQuery,
    >;

    /// Index auctions by end time.
    // map auction execution block number and auction id to auction
    #[pallet::storage]
    #[pallet::getter(fn auction_end_time)]
    pub(super) type AuctionsExecutionQueue<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        T::BlockNumber,
        Blake2_128Concat,
        u64,
        AuctionData<T::AccountId, T::BlockNumber, Bid<T::AccountId>, Tier>,
        OptionQuery,
    >;

    /////////////////////
    // Genesis config //
    ////////////////////
    // define pallet's genesis configuration
    #[pallet::genesis_config]
    pub struct GenesisConfig {
        pub auction_index: u64,
    }

    // assign default value for storage items
    #[cfg(feature = "std")]
    impl Default for GenesisConfig {
        fn default() -> Self {
            Self {
                auction_index: Default::default(),
            }
        }
    }

    // assign custom values at genesis block
    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig {
        fn build(&self) {
            // custom values are added here at the genesis block
            <AuctionIndex<T>>::put(&self.auction_index);
        }
    }

    ///////////////////
    // Pallet hooks //
    //////////////////
    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
        fn on_initialize(_now: T::BlockNumber) -> Weight {
            // T::WeightInfo::on_finalize(AuctionsExecutionQueue::<T>::iter_prefix(now).count() as u32)
            100_000_000.into()
        }

        fn on_finalize(now: T::BlockNumber) {
            // get auction ready for execution
            for (auction_id, _) in AuctionsExecutionQueue::<T>::drain_prefix(now) {
                if let Some(auction) = Auctions::<T>::take(auction_id) {
                    // handle auction execution
                    Self::on_auction_ended(auction.auction_id);
                }
            }
        }
    }

    //////////////////////
    // Runtime events  //
    /////////////////////
    // runtime event for important runtime actions
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        AuctionCreated {
            auction_id: u64,
            seller_id: T::AccountId,
            energy_quantity: u128,
            starting_price: u128,
        },

        AuctionBidAdded {
            auction_id: u64,
            seller_id: T::AccountId,
            energy_quantity: u128,
            bid: Bid<T::AccountId>,
        },

        AuctionMatched {
            auction_id: u64,
            seller_id: T::AccountId,
            energy_quantity: u128,
            starting_price: u128,
            highest_bid: Bid<T::AccountId>,
            matched_at: T::BlockNumber,
        },

        AuctionExecuted {
            auction_id: u64,
            seller_id: T::AccountId,
            buyer_id: T::AccountId,
            energy_quantity: u128,
            starting_price: u128,
            highest_bid: u128,
            executed_at: T::BlockNumber,
        },

        AuctionCanceled {
            auction_id: u64,
            seller_id: T::AccountId,
            energy_quantity: u128,
            starting_price: u128,
        },
    }

    //////////////////////
    // Pallet errors   //
    /////////////////////
    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        AuctionDoesNotExist,

        AuctionIsOver,

        InsuffficientAttachedDeposit,
    }

    ///////////////////////////
    // Pallet extrinsics    //
    //////////////////////////
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(100_000_000)]
        pub fn new(
            origin: OriginFor<T>,
            energy_quantity: u128, // in KWH
            starting_price: u128,  // in parachain native token
            auction_period: u16,   // in minutes
        ) -> DispatchResult {
            // Check that the extrinsic was signed by seller or return error.
            let seller = ensure_signed(origin)?;

            // get current_auction_id
            let current_auction_id = AuctionIndex::<T>::get().unwrap_or_default();

            // Calculate auction period
            // convert minutes to seconds and
            // divide by 6 (assumming each blocktime is 6 seconds)
            let auction_period_in_block_number = (auction_period.checked_mul(60).unwrap())
                .checked_div(6)
                .unwrap()
                .into();

            // Get current block number from the FRAME System pallet.
            let starting_block_number = <frame_system::Pallet<T>>::block_number();

            let ending_block_number = starting_block_number + auction_period_in_block_number;

            // Create starting bid
            let starting_bid = Bid::<T::AccountId> {
                bidder: seller.clone(),
                bid: starting_price,
            };

            // Categorize auction
            let category;
            if energy_quantity < 5 {
                category = Tier::default()
            } else {
                category = Tier { level: 2 }
            }

            // Create auction data
            let auction_data = AuctionData {
                auction_id: current_auction_id,
                seller_id: seller.clone(),
                quantity: energy_quantity,
                starting_bid: starting_bid.clone(),
                bids: vec![],
                auction_period: auction_period_in_block_number,
                auction_status: AuctionStatus::default(),
                start_at: starting_block_number,
                end_at: ending_block_number,
                highest_bid: starting_bid,
                auction_category: category,
            };

            // Get seller's auction information
            let mut seller_auction_info = AuctionsOf::<T>::get(seller.clone()).unwrap_or_default();

            // Ensure cached autions are less than 5
            // remove oldest auction
            if seller_auction_info.auctions.len() > 5 {
                seller_auction_info.auctions.pop();
            }

            // Update seller's auctions
            seller_auction_info.auctions.push(auction_data.auction_id);

            // Store seller's auction into storage
            seller_auction_info = AuctionInfo {
                participant_id: Some(seller.clone()),
                party_type: PartyType::Seller,
                auctions: seller_auction_info.auctions,
            };
            AuctionsOf::<T>::insert(&seller, seller_auction_info);

            // Add auction to execution queue
            AuctionsExecutionQueue::<T>::insert(
                auction_data.end_at,
                auction_data.auction_id,
                auction_data.clone(),
            );

            // Store globalauction to storage
            Auctions::<T>::insert(&auction_data.auction_id, auction_data.clone());

            // update auction id
            AuctionIndex::<T>::set(Some(current_auction_id + 1));

            // Emit an event that the auction was created.
            Self::deposit_event(Event::AuctionCreated {
                auction_id: auction_data.auction_id,
                seller_id: seller,
                energy_quantity,
                starting_price,
            });

            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(100_000_000)]
        pub fn cancel(origin: OriginFor<T>, auction_id: u64) -> DispatchResult {
            // Check that the extrinsic was signed by seller or return error.
            let _signer = ensure_signed(origin)?;

            // Check auction is exist
            ensure!(
                Auctions::<T>::contains_key(auction_id),
                Error::<T>::AuctionDoesNotExist
            );

            // Get auction from global auction
            let mut auction_data = Auctions::<T>::get(auction_id).unwrap();

            // Check auction is live
            ensure!(
                matches!(auction_data.auction_status, AuctionStatus::Open),
                Error::<T>::AuctionIsOver
            );

            // Close auction
            auction_data.auction_status = AuctionStatus::Closed;

            // Remove auction from global auctions
            Auctions::<T>::remove(auction_data.auction_id);

            // Get seller's auction info
            let mut sellers_auction_info =
                AuctionsOf::<T>::get(auction_data.seller_id.clone()).unwrap();

            // Remove auction from seller's auctions
            for (index, auction) in sellers_auction_info
                .auctions
                .clone()
                .into_iter()
                .enumerate()
            {
                // get matching auction(s)
                if auction_id == auction {
                    sellers_auction_info.auctions.remove(index);
                }
            }

            // update seller auction info
            AuctionsOf::<T>::insert(auction_data.seller_id.clone(), sellers_auction_info);

            // Remove auction from execution queue
            AuctionsExecutionQueue::<T>::remove(auction_data.end_at, auction_data.auction_id);

            // Emit an event that the auction was canceled.
            Self::deposit_event(Event::AuctionCanceled {
                auction_id: auction_data.auction_id,
                seller_id: auction_data.seller_id,
                energy_quantity: auction_data.quantity,
                starting_price: auction_data.starting_bid.bid,
            });

            Ok(())
        }

        #[pallet::call_index(2)]
        #[pallet::weight(100_000_000)]
        pub fn bid(origin: OriginFor<T>, auction_id: u64, bid: u128) -> DispatchResult {
            // Check that the extrinsic was signed by buyer or return error.
            let buyer_id = ensure_signed(origin)?;

            // Check auction is exist
            ensure!(
                Auctions::<T>::contains_key(auction_id),
                Error::<T>::AuctionDoesNotExist
            );

            // Get auction from global auction
            let mut auction_data = Auctions::<T>::get(auction_id).unwrap();

            // Check auction is live
            ensure!(
                matches!(auction_data.auction_status, AuctionStatus::Open),
                Error::<T>::AuctionIsOver
            );

            // Create new bid
            let new_bid = Bid::<T::AccountId> {
                bidder: buyer_id.clone(),
                bid,
            };

            // check if bid is highest bid
            if new_bid.bid > auction_data.bids.first().unwrap().bid {
                // add to top of auction bids
                auction_data.bids.insert(0, new_bid.clone());
            }

            // get buyer's auction information
            let buyer_auction_info = AuctionsOf::<T>::get(buyer_id.clone());

            match buyer_auction_info {
                // if buyer info already initialized, update info
                Some(mut auction_info) => {
                    for (index, auction) in auction_info.auctions.clone().into_iter().enumerate() {
                        // Ensure auction is within limit
                        if auction_info.auctions.len() >= 5 {
                            auction_info.auctions.pop();
                        }

                        // get matching auction
                        if auction == auction_id {
                            // insert new auction
                            auction_info.auctions.insert(index, auction_data.auction_id);

                            // update runtime storage
                            AuctionsOf::<T>::insert(
                                &buyer_id,
                                AuctionInfo {
                                    participant_id: Some(buyer_id.clone()),
                                    party_type: PartyType::Seller,
                                    auctions: auction_info.auctions.clone(),
                                },
                            )
                        }
                    }
                }
                // initialized and update information
                None => {
                    // Assign default information
                    let mut auction_info =
                        AuctionsOf::<T>::get(buyer_id.clone()).unwrap_or_default();

                    // Add auction to buyers information
                    auction_info.auctions.push(auction_data.auction_id);

                    // update runtime storage
                    AuctionsOf::<T>::insert(
                        &buyer_id,
                        AuctionInfo {
                            participant_id: Some(buyer_id.clone()),
                            party_type: PartyType::Seller,
                            auctions: auction_info.auctions.clone(),
                        },
                    )
                }
            }

            // Get seller's auction information
            let mut seller_auction_info =
                AuctionsOf::<T>::get(auction_data.clone().seller_id).unwrap();

            // Update seller's auction information
            for (index, auction) in seller_auction_info.auctions.clone().into_iter().enumerate() {
                // Ensure auction is within limit
                if seller_auction_info.auctions.len() >= 5 {
                    seller_auction_info.auctions.pop();
                }

                // get matching auction
                if auction == auction_id {
                    // insert new auction
                    seller_auction_info
                        .auctions
                        .insert(index, auction_data.auction_id);

                    // update runtime storage
                    AuctionsOf::<T>::insert(
                        &buyer_id,
                        AuctionInfo {
                            participant_id: Some(buyer_id.clone()),
                            party_type: PartyType::Seller,
                            auctions: seller_auction_info.auctions.clone(),
                        },
                    )
                }
            }

            // Update global auction
            Auctions::<T>::insert(&auction_data.auction_id, auction_data.clone());

            // Emit an event that the bid was created.
            Self::deposit_event(Event::AuctionBidAdded {
                auction_id: auction_data.auction_id,
                seller_id: auction_data.seller_id,
                energy_quantity: auction_data.quantity,
                bid: new_bid,
            });

            Ok(())
        }
    }

    ///////////////////////
    /// auction handler //
    //////////////////////
    impl<T: Config> Pallet<T> {
        fn on_auction_ended(auction_id: u64) {
            // Get auction data
            let auction_data = Auctions::<T>::get(auction_id).unwrap();
            let now = <frame_system::Pallet<T>>::block_number();

            // emit event that auction is matched
            Self::deposit_event(Event::AuctionMatched {
                auction_id: auction_data.auction_id,
                seller_id: auction_data.seller_id.clone(),
                energy_quantity: auction_data.quantity,
                starting_price: auction_data.starting_bid.bid,
                highest_bid: auction_data.highest_bid.clone(),
                matched_at: now,
            });

            // -------------Payment logic to be added here

            // emit evnt that auction has be executed
            Self::deposit_event(Event::AuctionExecuted {
                auction_id: auction_data.auction_id,
                seller_id: auction_data.seller_id,
                buyer_id: auction_data.highest_bid.bidder,
                energy_quantity: auction_data.quantity,
                starting_price: auction_data.starting_bid.bid,
                highest_bid: auction_data.highest_bid.bid,
                executed_at: now,
            });
        }
    }
}
