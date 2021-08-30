GET
curl -X GET -H "Content-Type:application/json" localhost:8080/api/status/order/{table_id}/{item}

Response:
{
	"timestamp": "xxxxxxxxxxxxx",
	"table_id": "4",
	"item": "B",
	"status": "?"
}

GET
curl -X GET -H "Content-Type:application/json" localhost:8080/api/status/order/{table_id}

Response:
{
	"timestamp": "xxxxxxxxxxxxx",
	"table_id": "3",
	"items": [
		{ "item": "A", "amount": "2", "status": "?" },
		{ "item": "B", "amount": "2", "status": "?" },
		{ "item": "C", "amount": "2", "status": "?" },
		{ "item": "D", "amount": "2", "status": "?" },
		...
		{ "item": "Z", "amount": "2", "status": "?" },
	]
}

POST
curl -X POST -H "Content-Type:application/json" localhost:8080/api/place/order -d "{\"timestamp\":123,\"table_id\":\"4\",\"items\": [{\"name\":\"A\", \"amount\":1}, {\"name\":\"B\", \"amount\":3}]}"

Request:
{
	"timestamp": "xxxxxxxxxxxx",
	"table_id":"4",
	"items": [
		{"name":"A", "amount":"1"}
	]
}

{
	"timestamp": "xxxxxxxxxxxx",
	"table_id":"4",
	"items": [
		{"name":"A", "amount":"1"},
i		{"name":"B", "amount":"3"}
	]
}

Response: {
	"timestamp": "xxxxxxxxxxxxx",
	"table_id": "6",
	"items": [
		{"name":"F", amount: "4", "status": "?"}
		{"name":"G", amount: "4", "status": "?"}
		...
		{"name":"H", amount: "4", "status": "?"}
	]
}

DELETE
curl -X DELETE -H "Content-Type:application/json" localhost:8080/api/delete/order -d "{\"timestamp\": 123,\"table_id\": \"4\", \"item\": \"D\"}"

Response: {
	"timestamp": "xxxxxxxxxxxxx",
	"table_id": "6",
	"items": [
		{"name":"F", amount: "0", "status": "?"}
	]
}

PUT
curl -X PUT -H "Content-Type:application/json" localhost:8080/api/update/order -d "{\"timestamp\":123,\"table_id\":\"4\",\"items\": [{\"name\":\"A\", \"amount\":1}, {\"name\":\"B\", \"amount\":3}]}"

Response: {
	"timestamp": "xxxxxxxxxxxxxx",
	"table_id": "6",
	"items": [
		{"name":"F", amount: "4", "status": "?"}
		{"name":"G", amount: "4", "status": "?"}
		...
		{"name":"H", amount: "4", "status": "?"}
	]
}

Client Behavior:
1. Send a req/sec based on randome number (0:place, 1:delete, 2:update, 3:status)
2. at least 10 clients (multithreading)
3. need a config file to determine 
	(1) # of clients spawned
	(2) cycle of req

Server Behavior:
1. LB machanism: own a concurrent queue to synchronize the received requests
2. listen on localhost:8080 when launched before requests comes in
3. db connection; record on each request status (place/delete/update/status)
4. failover machanism 
	(1) when a processor cannot handle volumes that exceed its capability, try direct flows to other proxies
	(2) when a processor crashed due to an unexpected reason, switch to other proxies to keep serving
5. 4 processors to deal with requests
6. graceful shutdown on each request
7. graceful shutdown on LB

DB Table Schema:
(
	timestamp LONG,
	table_id INT,
	table_status		// todo, doing, done
)

DB Items Schema:
(
	timestamp LONG,
	table_id INT,
	item String,
	amount INT,
	item_status Enum,	// todo, doing, done
	cook_time INT
)

Table <------>*Items