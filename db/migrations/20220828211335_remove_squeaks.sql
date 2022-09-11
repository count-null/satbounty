-- Add migration script here
alter table adminsettings
drop column squeaknode_address;

alter table adminsettings
drop column squeaknode_pubkey;

alter table usersettings
drop column squeaknode_address;

alter table usersettings
drop column squeaknode_pubkey;