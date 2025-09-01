## Server
### state
    Temp:
        [X] - onlines users
        [X] - unrad rooms
    DB: rooms
        [X] - rooms
        [X] - user, relation


### register auth
    [X] - apikey
    [X] - get user by api key
    [X] - register

### eventstream for listening to room events
    [X] - stream messages for rooms
    [X] - send message

### Other
    [x] error handleing
    [ ] sign key secrets store?
    [X] user guard
    [x] profile
    [x] guard database for token validation https://api.rocket.rs/master/rocket/request/trait.FromRequest
    [X] - auth key management
    [ ] - security check
    [X] - share entites by a library
    [ ] - folder structures
    [ ] - 404 not working??
    [X] - warnings[]

## Client
### Room
    [X]- previous 20 messages
    [x]- separate users chat from own
    [x]- scroll for messages
    [X]- sender of each message
    [x]- user profile
    [x] - logout

    [x] create new room
    [X]- date of each message
    [X]- right or left messages

### Profile section
    settings
    user profile
    
### Errors and loadings
    [ ]-retry

### Connect server to client
    [X]-auth token persistence?