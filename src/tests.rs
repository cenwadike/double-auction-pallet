#[allow(unused_imports)]
use crate::{mock::*, Error, Event};
// use frame_support::{assert_noop, assert_ok};

#[test]
fn create_new_auction_should_work() {
    new_test_ext().execute_with(|| {
        // initialize new auction params

        // dispatch signed extrinsic

        // assert that auction was added to auctions

        // assert that auction was added for user

        // assert that auction is in auction queue

        // assert that correct event was emitted
    });
}

#[test]
fn bid_should_work() {
    new_test_ext().execute_with(|| {
        // dispatch new auction extrinsic

        // initialize bid params

        // dispatch signed extrinsic for bid

        // assert that bid was added to the auction

        // assert that bid was added for buyer and seller

        // assert that correct event was emitted
    });
}

#[test]
fn cancel_auction_should_work() {
    new_test_ext().execute_with(|| {
        // dispatch new auction extrinsic

        // initialize cancel auction params

        // dispatch signed extrinsic for cancel auction

        // assert that auction was removed from auctions

        // assert that auction was removed from user

        // assert that auction is not in auction queue

        // assert that correct event was emitted
    });
}

#[test]
fn on_auction_ended_should_work() {
    new_test_ext().execute_with(|| {
        // dispatch new auction extrinsic

        // fast forward block production to auction end block height

        // assert that auction is not in auction queue

        // assert that correct event was emitted
    });
}
