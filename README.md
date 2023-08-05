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
     AuctionData {
        seller_id: AccountId,
        quantity: u128,
        starting_bid: u128,
        buyers: [], // Highest bidder at the top of the array.
        auction_period: Blockheight,
        start_at: Blockheight,
        ended_at: Blockheight,
    }
```

- All auctions
```rust
    Auctions<T: Config> = StorageMap<
        _,
        Twox64Concat,
        T::Hash,
        AuctionData<T::AccountId, T::BlockNumber, Bid<T::AccountId>, Tier>,
        OptionQuery,
    >
```

- Auction related to an origin 
```rust
    AuctionLookup<T: Config> = StorageMap<
        _,
        Twox64Concat,
        T::AccountId,
        AuctionsOf<T::AccountId, T::BlockNumber, Bid<T::AccountId>, Tier, PartyType>,
        OptionQuery,
    >
```

### `Interface:`
- create_auction(...)
- bid_auction(...)
- cancel_auction(...)

### `Hooks:`
    -- on_auctions_created
    -- on_auction_destroyed
    -- on_bid_auction
    -- on_auction_over

### `RPC:` 
- Data RPCs


Trait `AuctionHandler` is been used to validate the bid and when the auction ends `AuctionHandle::on_auction_ended(id, bid)` gets called.
