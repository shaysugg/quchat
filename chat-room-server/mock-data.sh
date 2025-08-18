#!/bin/bash
database_url=db/db.sqlite
room_ids=(
    "123e4567-e89b-12d3-a456-426614174000"
    "f47ac10b-58cc-4372-a567-0e02b2c3d479"
    "550e8400-e29b-41d4-a716-446655440000"
    "6ba7b810-9dad-11d1-80b4-00c04fd430c8"
    "9b8f7da0-e346-4c7f-a5b3-1f3d8f2a5c9d"
    "2c5ea4c0-d3d5-11e7-9296-cec278b6b50a"
    "1b671a64-40d5-491e-8b0c-82b7ede7f615"
    "5f1c3b5d-a2a1-4a3b-9e5f-7a3b2c1d4e5f"
    "8f14e45f-ceea-4165-ba76-4058f21a4f9b"
    "d1a5e6c0-3b1a-4f9d-8b2a-7c3d5e6f4a1b"
    "3f2504e0-4f89-11d3-9a0c-0305e82c3301"
    "6f9619ff-8b86-4bcc-9b90-363d5b3c5c3c"
    "a3e56f4c-1b2d-4a7e-9c3b-5d6f7a8b9c0d"
    "e2f34a5b-6c7d-11e8-9b6a-0800200c9a66"
    "1c3b5d6f-7a8b-9c0d-1e2f-3a4b5c6d7e8f"
    "9d8c7b6a-5f4e-3d2c-1b0a-0f9e8d7c6b5a"
    "4a3b2c1d-5e6f-7a8b-9c0d-1e2f3a4b5c6d"
    "7c3d5e6f-4a1b-2c3d-5e6f-7a8b9c0d1e2f"
    "b2a3c4d5-e6f7-8a9b-0c1d-2e3f4a5b6c7d"
    "5d6f7a8b-9c0d-1e2f-3a4b-5c6d7e8f9a0b"
    )

room_names=( 
    "luna"
    "zara"
    "kai"
    "finn"
    "nova"
    "arlo"
    "sage"
    "milo"
    "iris"
    "jett"
    "remi"
    "eden"
    "ash"
    "kira"
    "rio"
    "nora"
    "leo"
    "ivy"
    "max"
    "zoe"
    )
time_stamps=(
    1701388800
    1701475200
    1701561600
    1701648000
    1701734400
    1701820800
    1701907200
    1701993600
    1702080000
    1702166400
    1702252800
    1702339200
    1702425600
    1702512000
    1702598400
    1702684800
    1702771200
    1702857600
    1702944000
    1703030400
)
messages=(
    "hello world"
    "quick fox"
    "blue sky"
    "coding fun"
    "tech magic"
    "data flow"
    "swift code"
    "green leaf"
    "soft breeze"
    "night star"
    "calm river"
    "sharp mind"
    "pure logic"
    "wild dream"
    "brave path"
    "clear code"
    "smart move"
    "deep think"
    "fast track"
    "bold step"
)
messages_ids=(
  "e7e2baf2-71f3-44e6-9bb3-cd95f5c3b7c3"
  "a3d4b957-05cc-4983-9d87-2e4724f30b2c"
  "c59ae4a7-139b-42d2-93d3-3d2c8773bbcd"
  "8fcf4646-b9f0-4601-b3d0-c6c63702fffd"
  "2b86bd98-5c4a-472c-b97e-6c7cc51e1e41"
  "edc52c3d-464e-43a1-9c68-6f72b6764457"
  "9d52ac52-059b-4a18-8d44-3290c64f8268"
  "fb0c7aa1-59d2-4655-bc25-599e84516050"
  "75f1d9e6-d216-4a70-9f77-842bf33f5e52"
  "bea48ed4-e71d-4f4f-a8c1-8f1d6e6e4c21"
  "d4522e91-bf49-4b2f-8af1-2f26dba13b20"
  "6cfcae2e-c7b3-4070-bd4f-1915e00908ec"
  "a66a8d56-0b2e-4a0f-a635-2e5805c3a76e"
  "8d63b3ae-9112-4298-b179-fdbdf93aa2d1"
  "dd13dca6-2d06-45a6-aee4-07f31dce5a6e"
  "3c29bbf5-944e-4e83-b021-68e06340e3f3"
  "e5e4962e-f15a-4d59-84b0-9e8bc41c6db3"
  "8a6e376b-1ad9-489b-91b5-20ad5f4a28c9"
  "7cfeeb91-b7ea-407c-a271-7d2d8adac7c6"
  "50df8b43-2b84-455f-80b4-e86746f84e03"
  "f7f66a0a-08e7-41a7-9359-ec4e240ec242"
  "3e2c7be4-1890-40aa-896f-7e81c7c63e3a"
  "97dd3909-3762-4b93-9c3b-6d7e7fca63d4"
  "2e32a20b-b40a-4d0d-85b6-20568c8a3f24"
  "1cf39592-f6bc-498a-b430-92a3c3283837"
  "2673fa44-3314-4a3f-bb6e-64d9e1094c93"
  "b7852738-2d51-4b08-8108-07fcd49122a9"
  "f3625c89-5028-4c6e-9c83-661cb06194be"
  "779985a7-5cf2-4b5f-8a14-9175f8f8ab20"
  "f9f4a06a-78b6-464c-a07f-1c17c3d30c3e"
  "5291d8c7-1f7e-446e-a79c-727bc83b1529"
  "c2b4e32e-b645-4607-835f-7de97fa7f3f5"
  "5f8d17fc-5cd9-419c-96ee-bfb53e212541"
  "ed3042f6-c870-463e-b7f0-f772c82a3b38"
  "79d9cc13-92e5-4cf1-b239-1e09e9e7f41b"
  "fe4c92c3-ec4e-453d-b3c0-7c826489b1df"
  "9dc02ec4-dc83-4bbf-a0a1-0c585b3cda00"
  "f13cdce1-0171-44f3-b682-6ee26879f9d5"
  "f679f99c-fc43-4d91-a612-e3249e8f2961"
  "95d9bcef-bd1d-41c1-aee3-b6c2aa53dc6d"
)
user_id="5f6d05e8-06ad-40ef-a2d9-4bb734750dae"
for i in {0..20}
do
sqlite3 $database_url "INSERT INTO rooms (id, name, creator_id, create_date) VALUES ('${room_ids[i]}', '${room_names[i]}', '$user_id', ${time_stamps[i]})"
done

for i in {0..4}
do
for j in {0..10}
do
sqlite3 $database_url "INSERT INTO messages (id, content, room_id, sender_id, create_date) VALUES ('${messages_ids[i+j]}', '${messages[j]}', '${room_ids[i]}', '$user_id', ${time_stamps[i]})"
done
done