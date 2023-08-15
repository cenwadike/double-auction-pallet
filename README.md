# Double Auction module

## Overview

This module provides implementation for on-chain double auctioning.

This is the matching layer of a decentralized marketplace for electrical energy.
Sellers are categorized based on how much electricity they intend to sell.
Buyers are also categorized based on how much electricity they intend to buy.

The highest bidding buyer in the same category with a seller is matched
when the auction period of a seller is over.

The seller has the benefit of getting the best price at a given point in time for their category,
while the buyer can choose a margin of safety for every buy.

NOTE: this mocdule does not implement how payment is handled.

### `Data`:  

- Data relevant to an auction
```rust
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
```

- Infomation of a participant
```rust
    pub struct AuctionInfo<AccountId, BlockNumber, Bid, Tier, PartyType> {
        pub participant_id: Option<AccountId>,
        pub party_type: PartyType,
        pub auctions: Vec<AuctionData<AccountId, BlockNumber, Bid, Tier>>, // Maximum length of 5
    }
```

- All auctions
```rust
    pub(super) type Auctions<T: Config> = StorageMap<
        _,
        Twox64Concat,
        u64, // auction id
        AuctionData<T::AccountId, T::BlockNumber, Bid<T::AccountId>, Tier>,
        OptionQuery,
    >
```

- Auction related to an account id 
```rust
    pub(super) type AuctionsOf<T: Config> = StorageMap<
        _,
        Twox64Concat,
        T::AccountId,
        AuctionInfo<T::AccountId, T::BlockNumber, Bid<T::AccountId>, Tier, PartyType>,
        OptionQuery,
    >
```

- Auction execution queue
```rust
    pub(super) type AuctionsExecutionQueue<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        T::BlockNumber,
        Blake2_128Concat,
        u64,
        AuctionData<T::AccountId, T::BlockNumber, Bid<T::AccountId>, Tier>,
        OptionQuery,
    >
```

### `Interface:`
- new(...) [x]
- bid(...) [ ]
- cancel(...) []

### `Hooks:`
- on_auctions_created [ ]
- on_auction_destroyed [ ]
- on_bid_auction   [ ]
- on_auction_ended [x]

### `RPC:` 
- Data RPCs


Trait `AuctionHandler` is been used to validate the bid and when the auction ends `AuctionHandle::on_auction_ended(id, bid)` gets called.
