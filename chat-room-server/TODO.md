# TODO 
state
    Temp:
        [X] - onlines users
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
    [] error handleing
    [] sign key secrets store?
    [X] user guard
    [] profile
    [] guard database for token validation https://api.rocket.rs/master/rocket/request/trait.FromRequest
    [X] auth key management


//http POST http://localhost:8000/auth/logout -A bearer -a eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJ1c2VyX2lkIjoiNjgxZDk0ZjctYmU2Yy00ZmY5LWFjNmEtYzVlOTQzZDc3NzRhIiwiZXhwIjowfQ==.KOihGmc1SC6PQEqGogXap9pWE8S8Y6M1/LebSWGiUGE=