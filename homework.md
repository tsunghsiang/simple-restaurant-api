GET
curl -X GET -H "Content-Type:application/json" localhost:8080/api/status/order/{table_id}/{item}
curl -X GET -H "Content-Type:application/json" localhost:8080/api/status/order/{table_id}

POST
curl -X POST -H "Content-Type:application/json" localhost:8080/api/place/order -d "{\"table_id\":\"4\",\"items\": [{\"name\":\"A\", \"amount\":1}, {\"name\":\"B\", \"amount\":3}]}"

{
	"table_id":"4",
	"items": [
		{"name":"A", "amount":"1"}
	]
}

{
	"table_id":"4",
	"items": [
		{"name":"A", "amount":"1"},
		{"name":"B", "amount":"3"}
	]
}

DELETE
curl -X DELETE -H "Content-Type:application/json" localhost:8080/api/delete/order -d "{\"table_id\": \"4\", \"item\": \"D\"}"

PUT
curl -X PUT -H "Content-Type:application/json" localhost:8080/api/update/order -d "{\"table_id\":\"4\",\"items\": [{\"name\":\"A\", \"amount\":1}, {\"name\":\"B\", \"amount\":3}]}"