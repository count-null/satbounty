-- Add migration script here
ALTER TABLE bounties RENAME COLUMN reviewed TO viewed;
ALTER TABLE cases DROP COLUMN reviewed;
ALTER TABLE cases DROP COLUMN review_text;
ALTER TABLE cases DROP COLUMN review_rating;
ALTER TABLE cases DROP COLUMN review_time_ms;
