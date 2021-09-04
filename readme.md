# Simple Restaurant API
## Introduction
The project is developed for the [interview problem](https://github.com/paidy/interview/blob/master/SimpleRestaurantApi.md) of [Paidy Inc.](https://paidy.com/), in order to realize a simple food ordering system in a restaurant. The APIs aim to place/delete/update/query orders on demand with some added rules.
## REST API Specifications

| Description                                                                                                                                                                          | Method | Token (Y/N) |               path                |
| :----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | :----: | :---------: | :-------------------------------: |
| Show all items for a specified table number                                                                                                                                          |  GET   |      N      |    /api/status/order/:table_id    |
| Show a specified item for a specified table number                                                                                                                                   |  GET   |      N      | /api/status/order/:table_id/:item |
| Create a request: store item, table number and cooking period                                                                                                                        |  POST  |      Y      |         /api/place/order          |
| Delete a request: remove a specified item for a specified table number                                                                                                               | DELETE |      Y      |         /api/delete/order         |
| Update a request: for a created request not fully served, a staff is able to update amounts of specified items and add new items on the same order, but served items are not updated | PATCH  |      Y      |         /api/update/order         |

```table_id```: The identifier of a table, which is unique.

```item```: The name of the food. In our scenario, it is limited to upper-case alphabets: **(A, B, C, ... , X, Y, Z)**.

```Base URL```: localhost:8080

## Basic Testing Samples
Usually, you can test on your own by [curl](https://linux.die.net/man/1/curl) command
1. Get all items of a specified table number.
   
   ```curl -X GET -H "Content-Type:application/json" localhost:8080/api/status/order/3```
   
   Response JSON Format:
    ```json
    {
    	"timestamp": 1630726854764,
    	"table_id": "3",
    	"items": [
    		{ "item": "A", "amount": "2", "item_status": "todo" },
    		{ "item": "B", "amount": "2", "status": "todo" },
    		{ "item": "C", "amount": "2", "status": "doing" },
    		{ "item": "Z", "amount": "2", "status": "done" },
    	]
    }
    ```
1. Get status of a specified item of a specified table number.
   
   ```curl -X GET -H "Content-Type:application/json" localhost:8080/api/status/order/3/Z```
   
   Response JSON Format:
    ```json
    {
    	"timestamp": 1630726854764,
    	"table_id": "3",
    	"item": "Z",
    	"item_status": "done"
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