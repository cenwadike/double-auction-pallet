use crate::{mock::*, Event};
use frame_support::assert_ok;
use sp_runtime::AccountId32;

#[test]
fn create_new_auction_should_work() {
    new_test_ext().execute_with(|| {
        // go to block after genesis
        // genesis block does not emit event
        System::set_block_number(2);

        // initialize new auction params
        let seller = RuntimeOrigin::signed(AccountId::from(AccountId32::from(
            b"000000000000000000000ALICE000000".clone(),
        )));
        let energy_quantity = 2; // in KWH
        let starting_price = 1_000;
        let auction_period = 5; // in minutes

        let execution_block = System::block_number() + 50;

        // dispatch signed extrinsic
        assert_ok!(DoubleAuctionModule::new(
            seller,
            energy_quantity,
            starting_price,
            auction_period
        ));

        // assert that auction was added to auctions
        let auction = DoubleAuctionModule::auctions(0).expect("return indexed auction");

        assert_eq!(
            auction.seller_id,
            AccountId::from(AccountId32::from(
                b"000000000000000000000ALICE000000".clone(),
            ))
        );
        assert_eq!(auction.quantity, energy_quantity);
        assert_eq!(auction.starting_bid.bid, starting_price);

        // assert that auction was added for user
        let seller_auction_info = DoubleAuctionModule::auctions_of(AccountId::from(
            AccountId32::from(b"000000000000000000000ALICE000000".clone()),
        ))
        .expect("return seller's auction information");

        assert_eq!(
            seller_auction_info.participant_id.unwrap(),
            AccountId32::from(b"000000000000000000000ALICE000000".clone())
        );

        // assert that auction is in auction queue
        assert!(
            DoubleAuctionModule::auction_execution_queue(execution_block, auction.auction_id)
                .is_some()
        );

        // assert that correct event was emitted
        System::assert_has_event(RuntimeEvent::DoubleAuctionModule(Event::AuctionCreated {
            auction_id: auction.auction_id,
            seller_id: auction.seller_id,
            energy_quantity: auction.quantity,
            starting_price,
        }));
    })
}

#[test]
fn cancel_auction_should_work() {
    new_test_ext().execute_with(|| {
        // go to block after genesis
        // genesis block does not emit event
        System::set_block_number(2);

        // initialize new auction params
        let seller = RuntimeOrigin::signed(AccountId::from(AccountId32::from(
            b"000000000000000000000ALICE000000".clone(),
        )));
        let energy_quantity = 2; // in KWH
        let starting_price = 1_000;
        let auction_period = 5; // in minutes

        let execution_block = System::block_number() + 50;

        // dispatch new auction extrinsic
        assert_ok!(DoubleAuctionModule::new(
            seller.clone(),
            energy_quantity,
            starting_price,
            auction_period
        ));

        // assert that auction was added to auctions
        let auction = DoubleAuctionModule::auctions(0).expect("return indexed auction");

        // dispatch signed extrinsic for cancel auction
        assert_ok!(DoubleAuctionModule::cancel(
            seller.clone(),
            auction.auction_id
        ));

        // assert that auction was removed from auctions
        assert!(DoubleAuctionModule::auctions(auction.auction_id).is_none());

        // assert that auction was removed from user
        assert!(
            DoubleAuctionModule::auctions_of(AccountId::from(AccountId32::from(
                b"000000000000000000000ALICE000000".clone(),
            )))
            .unwrap()
            .auctions
            .get(auction.auction_id as usize)
            .is_none()
        );

        // println!(
        //     "{:?}",
        //     DoubleAuctionModule::auctions_of(AccountId::from(AccountId32::from(
        //         b"000000000000000000000ALICE000000".clone(),
        //     )))
        //     .unwrap()
        //     .auctions
        //     .get(auction.auction_id as usize)
        //     .unwrap()
        // );

        // assert that auction is not in auction queue
        assert!(
            DoubleAuctionModule::auction_execution_queue(execution_block, auction.auction_id)
                .is_none()
        );

        // assert that correct event was emitted
        System::assert_has_event(RuntimeEvent::DoubleAuctionModule(Event::AuctionCanceled {
            auction_id: auction.auction_id,
            seller_id: auction.seller_id,
            energy_quantity: auction.quantity,
            starting_price: auction.starting_bid.bid,
        }));
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
fn on_auction_ended_should_work() {
    new_test_ext().execute_with(|| {
        // dispatch new auction extrinsic

        // fast forward block production to a block after auction end block height

        // assert that auction is not in auction queue

        // assert that correct event was emitted
    });
}
