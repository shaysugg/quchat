#!/bin/bash
#/TODO
db_path="/Users/shayan/DEV/rust/chat-room/db-other/token_blacklist.sqlite"
db_url="sqlite:$db_path"

if [ -e $db_path ]; then 
rm $db_path
fi

touch $db_path

sqlx migrate run --source ./db-other --database-url $db_url
cargo sqlx prepare