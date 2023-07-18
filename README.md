# Double auction module

### Overview

Double auction module provides a way to match different open auction and place bids on-chain. 

You can open an auction by specifying a `start: BlockNumber` and/or an `end: BlockNumber`, and when the auction becomes active enabling anyone to place a bid at a higher price. 

You can also place a bid and get matched with the auction where you are the highest bidder.

Trait `AuctionHandler` is been used to validate the bid and when the auction ends `AuctionHandle::on_auction_ended(id, bid)` gets called.