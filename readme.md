# Simple Restaurant API
## Introduction
The project is developed for the [interview problem](https://github.com/paidy/interview/blob/master/SimpleRestaurantApi.md) of [Paidy Inc.](https://paidy.com/), in order to realize a simple food ordering system in a restaurant. The APIs aim to place/delete/update/query orders on demand with some added rules.
## REST API Specifications

| Description                                                                                                                                                                          | Method | Basic Auth (Y/N) |               path                |
| :----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | :----: | :--------------: | :-------------------------------: |
| Show all items for a specified table number                                                                                                                                          |  GET   |        N         |    /api/status/order/:table_id    |
| Show a specified item for a specified table number                                                                                                                                   |  GET   |        N         | /api/status/order/:table_id/:item |
| Create a request: store item, table number and cooking period                                                                                                                        |  POST  |        Y         |         /api/place/order          |
| Delete a request: remove a specified item for a specified table number                                                                                                               | DELETE |        Y         |         /api/delete/order         |
| Update a request: for a created request not fully served, a staff is able to update amounts of specified items and add new items on the same order, but served items are not updated | PATCH  |        Y         |         /api/update/order         |

```table_id```: The identifier of a table, which is unique.

```item```: The name of the food. In our scenario, it is limited to upper-case alphabets: **(A, B, C, ... , X, Y, Z)**.

```Base URL```: localhost:8080

## Basic Testing Samples
Usually, you can test on your own by [curl](https://linux.die.net/man/1/curl) command
1. **Get all items of a specified table number**.
   
   ```curl -X GET -H "Content-Type:application/json" localhost:8080/api/status/order/3```
   
   Response JSON Format:
    ```json
    {
    	"timestamp": 1630726854764,
    	"table_id": "3",
    	"items": [
    		{ "item": "A", "amount": 2, "item_status": "todo" },
    		{ "item": "B", "amount": 2, "item_status": "todo" },
    		{ "item": "C", "amount": 2, "item_status": "doing" },
    		{ "item": "Z", "amount": 2, "item_status": "done" },
    	]
    }
    ```
2. **Get status of a specified item of a specified table number**.
   
   ```curl -X GET -H "Content-Type:application/json" localhost:8080/api/status/order/3/Z```
   
   Response JSON Format:
    ```json
    {
    	"timestamp": 1630726854764,
    	"table_id": "3",
    	"item": "Z",
    	"amount": 1,
    	"item_status": "done"
    }
    ```
3. **Create a new request for aquiring items of a specific table**. In the example, we are going to order items {A, B, C} with amount {1, 2, 3} respectively, for table id 4.

    ```curl -X POST -H "Content-Type:application/json" -H "X-Auth-Username:{encrypted username}}" -H "X-Auth-Password:{encrypted password}" localhost:8080/api/place/order -d "{\"timestamp\":1234567890123,\"table_id\":\"4\",\"items\": [{\"name\":\"A\", \"amount\":1}, {\"name\":\"B\", \"amount\":2}, {\"name\":\"C\", "amount":3}]}"```
    
    Request JSON Format:
    ```json
    {
    	"timestamp": 1234567890123,
    	"table_id":"4",
    	"items": [
    		{"name":"A", "amount":1},
    		{"name":"B", "amount":2},
    		{"name":"C", "amount":3},
    	]
    }
    ```
4. **Remove an item from a list of a specific table id**.

    ```curl -X DELETE -H "Content-Type:application/json" -H "X-Auth-Username:{encrypted username}}" -H "X-Auth-Password:{encrypted password}" localhost:8080/api/delete/order -d "{\"timestamp\": 1234567890123,\"table_id\": \"4\", \"item\": \"A\"}"```

    Request JSON Format:
    ```json
    {
        "timestamp": 1234567890123,
        "table_id": "4",
        "item": "A"
    }
    ```
5. **Update one or more items of a specific table**. For example, if you'd like to update the amount of an item that is neither done nor doing.

    ```curl -X PATCH -H "Content-Type:application/json" -H "X-Auth-Username:{encrypted username}}" -H "X-Auth-Password:{encrypted password}" localhost:8080/api/update/order -d "{\"timestamp\":1234567890123,\"table_id\":\"4\",\"items\": [{\"name\":\"A\", \"amount\":8}, {\"name\":\"B\", \"amount\":3}]}"```

    Request JSON Format:
    ```json
    {
        "timestamp": 1234567890123,
        "table_id": "4",
        "items": [
            { "name": "A", amount: 8 },
            { "name": "B", amount: 3 }
        ]
    }
    ```

## Order Rules
## Prerequisites
- DB version
- Configurations
## DB Schema Design
## Build Server/Client
## Start Running Server/Client
## Other Issues