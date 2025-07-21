-- Add up migration script here
CREATE TABLE IF NOT EXISTS rooms (
    id TEXT NOT NULL PRIMARY KEY,
    name TEXT NOT NULL DEFAULT "",
    creator_id TEXT NOT NULL,
    create_date INT NOT NULL
);

CREATE TABLE IF NOT EXISTS users (
    id TEXT NOT NULL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    secret TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS messages (
    id TEXT NOT NULL PRIMARY KEY,
    content TEXT NOT NULL,
    room_id TEXT NOT NULL,
    sender_id TEXT NOT NULL,
    create_date INT NOT NULL
);