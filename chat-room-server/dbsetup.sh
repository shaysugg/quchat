#!/bin/bash

# change this base on env
db_path="$(pwd)/db/db.sqlite"
db_url="sqlite://$db_path"

if [ -e $db_path ]; then 
rm $db_path
fi

touch $db_path


sqlx migrate run --source ./db --database-url $db_url
cargo sqlx prepare --database-url $db_url