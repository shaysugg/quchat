for num in {0..10}
do
http http://localhost:8000/rooms name="room ${num}" -A bearer -a $1;
done


