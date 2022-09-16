-- Add migration script here
ALTER TABLE listings RENAME TO bounties;
ALTER TABLE listingimages RENAME TO bountyimages;

ALTER TABLE bountyimages RENAME COLUMN listing_id TO bounty_id;
ALTER TABLE orders RENAME COLUMN listing_id TO bounty_id;
