# TODO 
state
    Temp:
        [X] - onlines users
        [ ] - unred rooms
    DB: rooms
        [X] - rooms
        [X] - user, relation


register auth
    [X] - apikey
    [X] - get user by api key
    [X] - register

eventstream for listening to room events
    [X] - stream messages for rooms
    [X] - send message

Other
    [x] error handleing
    [ ] sign key secrets store?
    [X] user guard
    [x] profile
    [x] guard database for token validation https://api.rocket.rs/master/rocket/request/trait.FromRequest
    [X] auth key management
    [ ] security check
    [ ] share entites by a library
    [ ] folder structures
    [ ] 404 not working??

SELECT * FROM room_state WHERE room_id IN ($1) AND user_id = ($2)