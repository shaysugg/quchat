-- Add down migration script here
DROP TABLE IF EXISTS rooms;
DROP TABLE IF EXISTS users;
DROP TABLE IF EXISTS messages;
DROP TABLE IF EXISTS token_blacklist;
DROP TABLE IF EXISTS room_state;