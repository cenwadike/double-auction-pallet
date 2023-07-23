//! # Auction
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
//! NOTE: this mocdule does not implement how payment is handled.
//!
//! `Data`:     
//!     --  AuctionData {
//!             seller_id: AccountId,
//!             quantity: u128,
//!             starting_bid: u128,
//!             buyers: [], // sorted array of bidders according to bid. Highest bidder at the top of the array.
//!             auction_period: Blockheight,
//!             start_at: Blockheight,
//!             ended_at: Blockheight,
//!         }
//!     -- Tier: u128,  // 0, 1, 2, ...
//!     -- Auctions {map(hash(AuctionData + Salt) -> (AuctionData, AuctionCategory, Tier)}
//!
//! `Interface`:
//!     -- create_auction(...)
//!     -- bid_auction(...)
//!     -- cancel_auction(...)
//!
//! `Hooks`:
//!     -- on_auctions_created
//!     -- on_auction_destroyed
//!     -- on_bid_auction
//!     -- on_auction_over
//!
//! `RPC`: Data RPCs

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
    use frame_support::sp_runtime::traits::Hash;
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
        bidder: AccountId,
        bid: u128,
    }

    // Status of an auction, live auctions accepts bids
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
    pub enum AuctionStatus {
        Alive,
        Dead,
    }
    impl Default for AuctionStatus {
        fn default() -> Self {
            AuctionStatus::Alive
        }
    }

    // Essential data for an auction
    #[derive(Clone, Encode, Decode, Default, Eq, PartialEq, RuntimeDebug, TypeInfo)]
    pub struct AuctionData<AccountId, BlockNumber, Bid, Tier> {
        pub auction_id: u64,
        pub seller_id: AccountId,
        pub quantity: u128,
        pub starting_bid: u128,
        pub memo: Vec<u8>,
        bids: Vec<Bid>,
        auction_period: BlockNumber,
        auction_status: AuctionStatus,
        start_at: BlockNumber,
        ended_at: BlockNumber,
        highest_bid: Bid,
        auction_category: Tier,
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
    // for quick data retrieval
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
    pub struct AuctionsOf<AccountId, BlockNumber, Bid, Tier, PartyType> {
        pub party: Option<AccountId>,
        pub party_type: PartyType,
        pub auctions: Vec<AuctionData<AccountId, BlockNumber, Bid, Tier>>, // Maximum length of 10
    }

    impl<AccountId, BlockNumber> Default
        for AuctionsOf<AccountId, BlockNumber, Bid<AccountId>, Tier, PartyType>
    {
        fn default() -> Self {
            AuctionsOf {
                party: None,
                party_type: PartyType::Seller,
                auctions: vec![],
            }
        }
    }

    //////////////////////
    // Storage item    //
    /////////////////////
    #[pallet::storage]
    #[pallet::getter(fn current_auction_id)]
    pub(super) type AuctionId<T: Config> = StorageValue<_, u64>;

    // live auctions
    #[pallet::storage]
    #[pallet::getter(fn get_auction)]
    pub(super) type Auctions<T: Config> = StorageMap<
        _,
        Twox64Concat,
        T::Hash,
        AuctionData<T::AccountId, T::BlockNumber, Bid<T::AccountId>, Tier>,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn get_auction_of)]
    pub(super) type AuctionLookup<T: Config> = StorageMap<
        _,
        Twox64Concat,
        T::AccountId,
        AuctionsOf<T::AccountId, T::BlockNumber, Bid<T::AccountId>, Tier, PartyType>,
        OptionQuery,
    >;

    //////////////////////
    // Runtime events  //
    /////////////////////
    // runtime event for important runtime actions
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        AuctionCreated {
            seller: T::AccountId,
            energy_quantity: u128,
            starting_price: u128,
            memo: Vec<u8>,
        },

        AuctionBidAdded {
            seller: T::AccountId,
            energy_quantity: u128,
            memo: Vec<u8>,
            bidder: T::AccountId,
            bid: u128,
        },

        AuctionMatched {
            seller: T::AccountId,
            buyer: T::AccountId,
            energy_quantity: u128,
            starting_price: u128,
            highest_bid: u128,
            matched_at: T::BlockNumber,
        },

        AuctionExecuted {
            seller: T::AccountId,
            buyer: T::AccountId,
            energy_quantity: u128,
            starting_price: u128,
            highest_bid: u128,
            executed_at: T::BlockNumber,
        },

        AuctionCanceled {
            seller: T::AccountId,
            energy_quantity: u128,
            starting_price: u128,
            memo: Vec<u8>,
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

        UnAuthorizedCall,

        InsuffficientAttachedDeposit,
    }

    ///////////////////
    // Pallet hooks //
    //////////////////

    ///////////////////////////
    // Pallet extrinsics    //
    //////////////////////////
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(100_000_000)]
        pub fn create_auction(
            origin: OriginFor<T>,
            energy_quantity: u128, // in KWH
            starting_price: u128,  // in parachain native token
            auction_period: u16,   // in minutes
            memo: Vec<u8>,         // additional info for indexing
        ) -> DispatchResult {
            // Check that the extrinsic was signed by seller or return error.
            let seller = ensure_signed(origin)?;

            // get current_auction_id
            let current_auction_id = AuctionId::<T>::get().unwrap_or(1);

            // Calculate auction period
            // Convert minutes to seconds
            // Divide by 6, assumming each time is 6 seconds
            let auction_period_in_block_number = (auction_period.checked_mul(60).unwrap())
                .checked_div(6)
                .unwrap()
                .into();

            // Get current block number from the FRAME System pallet.
            let starting_block_number = <frame_system::Pallet<T>>::block_number();

            let ending_block_number = starting_block_number + auction_period_in_block_number;

            let default_bid = Bid::<T::AccountId> {
                bidder: seller.clone(),
                bid: starting_price,
            };

            let category;
            if energy_quantity < 5 {
                category = Tier::default()
            } else {
                category = Tier { level: 2 }
            }

            let auction_data = AuctionData {
                auction_id: current_auction_id,
                seller_id: seller.clone(),
                quantity: energy_quantity,
                starting_bid: starting_price,
                memo: memo.clone(),
                bids: vec![],
                auction_period: auction_period_in_block_number,
                auction_status: AuctionStatus::default(),
                start_at: starting_block_number,
                ended_at: ending_block_number,
                highest_bid: default_bid,
                auction_category: category,
            };

            // get auction from lookup
            let mut auctions_of_seller =
                AuctionLookup::<T>::get(seller.clone()).unwrap_or_default();

            // remove least current auction from lookup auctions if length > 10
            if auctions_of_seller.auctions.len() > 10 {
                auctions_of_seller.auctions.pop();
            }

            // update lookup auctions
            auctions_of_seller.auctions.push(auction_data.clone());

            // Add seller's auctions to lookup map
            let auction_of_seller = AuctionsOf {
                party: Some(seller.clone()),
                party_type: PartyType::Seller,
                auctions: auctions_of_seller.auctions,
            };
            AuctionLookup::<T>::insert(&seller, auction_of_seller);

            // add auction to runtime storgae
            let pre_image = format!("{:?}.{:?}", seller.clone().encode(), memo);
            let auction_hash = T::Hashing::hash(pre_image.as_bytes());
            Auctions::<T>::insert(&auction_hash, auction_data);

            // update auction id
            AuctionId::<T>::set(Some(current_auction_id + 1));

            // Emit an event that the auction was created.
            Self::deposit_event(Event::AuctionCreated {
                seller,
                energy_quantity,
                starting_price,
                memo,
            });

            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(100_000_000)]
        pub fn cancel_auction(origin: OriginFor<T>, memo: Vec<u8>) -> DispatchResult {
            // Check that the extrinsic was signed by seller or return error.
            let seller = ensure_signed(origin)?;

            // Get auction hash
            let pre_image = format!("{:?}.{:?}", seller.clone().encode(), memo);
            let auction_hash = T::Hashing::hash(pre_image.as_bytes());

            // Check auction is exist
            ensure!(
                Auctions::<T>::contains_key(auction_hash),
                Error::<T>::AuctionDoesNotExist
            );

            // Get auction
            let mut auction_data = Auctions::<T>::get(auction_hash).unwrap();

            // Check auction is live
            ensure!(
                matches!(auction_data.auction_status, AuctionStatus::Alive),
                Error::<T>::AuctionIsOver
            );

            // get auction from lookup
            let mut auctions_of_seller = AuctionLookup::<T>::get(seller.clone()).unwrap();

            // check corresponding auction in lookup and update auction data
            for (index, mut auction) in auctions_of_seller.auctions.clone().into_iter().enumerate()
            {
                // get matching auction(s)
                if auction.memo == memo {
                    auction.auction_status = AuctionStatus::Dead;
                    auctions_of_seller.auctions.remove(index);
                    auctions_of_seller.auctions.insert(index, auction);
                }

                // update runtime storage
                AuctionLookup::<T>::insert(
                    &seller,
                    AuctionsOf {
                        party: Some(seller.clone()),
                        party_type: PartyType::Seller,
                        auctions: auctions_of_seller.auctions.clone(),
                    },
                )
            }

            // Set auction as over
            auction_data.auction_status = AuctionStatus::Dead;

            // update auction in runtime storage
            Auctions::<T>::insert(&auction_hash, auction_data.clone());

            // Emit an event that the auction was canceled.
            Self::deposit_event(Event::AuctionCanceled {
                seller: auction_data.seller_id,
                energy_quantity: auction_data.quantity,
                starting_price: auction_data.starting_bid,
                memo: auction_data.memo,
            });

            Ok(())
        }

        #[pallet::call_index(2)]
        #[pallet::weight(100_000_000)]
        pub fn bid_auction(
            origin: OriginFor<T>,
            seller_id: T::AccountId,
            auction_memo: Vec<u8>,
            bid: u128,
        ) -> DispatchResult {
            // Check that the extrinsic was signed by buyer or return error.
            let buyer = ensure_signed(origin)?;

            // Get auction hash
            let pre_image = format!("{:?}.{:?}", seller_id.clone().encode(), auction_memo);
            let auction_hash = T::Hashing::hash(pre_image.as_bytes());

            // Check auction is exist
            ensure!(
                Auctions::<T>::contains_key(auction_hash),
                Error::<T>::AuctionDoesNotExist
            );

            // Get auction data
            let mut auction_data = Auctions::<T>::get(auction_hash).unwrap();

            // Check auction is live
            ensure!(
                matches!(auction_data.auction_status, AuctionStatus::Alive),
                Error::<T>::AuctionIsOver
            );

            // add new bid
            let new_bid = Bid::<T::AccountId> {
                bidder: buyer.clone(),
                bid,
            };

            // check if bid is highest bid and add to top of auction bids
            if new_bid.bid > auction_data.clone().bids.first().unwrap().bid {
                auction_data.bids.insert(0, new_bid);
            }

            // get selller's auctiona from lookup
            let mut auctions_of_seller = AuctionLookup::<T>::get(seller_id.clone()).unwrap();

            // check corresponding auction in lookup for seller and update auction data
            for (index, auction) in auctions_of_seller.auctions.clone().into_iter().enumerate() {
                // get matching auction(s)
                if auction.memo == auction_memo {
                    auctions_of_seller
                        .auctions
                        .insert(index, auction_data.clone());
                }

                // update runtime storage
                AuctionLookup::<T>::insert(
                    &seller_id,
                    AuctionsOf {
                        party: Some(buyer.clone()),
                        party_type: PartyType::Seller,
                        auctions: auctions_of_seller.auctions.clone(),
                    },
                )
            }

            // Get auctions of buyer
            let mut auctions_of_buyer = AuctionLookup::<T>::get(buyer.clone()).unwrap_or_default();

            // remove least current auction from lookup auctions if length > 10
            if auctions_of_buyer.auctions.len() > 10 {
                auctions_of_buyer.auctions.pop();
            }

            // update buyers lookup auctions
            auctions_of_buyer.auctions.push(auction_data.clone());

            // Add buyer's auctions to lookup map runtime storage
            let auctions_of_buyer = AuctionsOf {
                party: Some(buyer.clone()),
                party_type: PartyType::Buyer,
                auctions: auctions_of_buyer.auctions,
            };
            AuctionLookup::<T>::insert(&buyer, auctions_of_buyer);

            // update auction in runtime storage
            Auctions::<T>::insert(&auction_hash, auction_data.clone());

            // Emit an event that the bid was created.
            Self::deposit_event(Event::AuctionBidAdded {
                seller: seller_id.clone(),
                energy_quantity: auction_data.quantity,
                memo: auction_memo,
                bidder: buyer,
                bid,
            });

            Ok(())
        }
    }
}
