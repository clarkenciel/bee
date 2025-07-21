-- Add up migration script here
create table if not exists words (
  word text primary key
  , letter_mask integer not null
  , length integer not null
);
