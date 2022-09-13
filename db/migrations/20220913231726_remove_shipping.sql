-- Add migration script here
drop table shippingoptions;

alter table orders drop column shipping_option_id;
alter table orders rename column shipping_instructions to case_details;
alter table orders rename column shipped to awarded;
