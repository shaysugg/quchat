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

db_path_other="$(pwd)/db-other/token_blacklist.sqlite"
db_url_other="sqlite://$(pwd)/db-other/token_blacklist.sqlite"


mkdir .sqlx-tmp
mv .sqlx/* .sqlx-tmp/

if [ -e $db_path_other ]; then 
rm $db_path_other
fi

touch $db_path_other

sqlx migrate run --source ./db-other --database-url $db_url_other
cargo sqlx prepare --database-url $db_url_other

mv .sqlx-tmp/* .sqlx/

rm -rf .sqlx-tmp