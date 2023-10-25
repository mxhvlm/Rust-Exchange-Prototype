# Rust-Exchange-Prototype

is a rust-based prototype for an order book and matching engine with that leverages efficient datastructures to process a large number of transactions per second, even for big orderbooks.

The orderbook consists of a set of 'pages' - one for each discrete price level. Each page contains a linked set of all the orders sitting at the page's price.


## Time Complexity

### Inserting an Order
1. Find the appropriate page in the B-Tree, which takes O(log n) time where n is the number of price levels.
2. Insert the order into the LinkedHashMap at that price level. Since we're inserting at the end, it takes O(1) time.
3. Finally, we also add the order to your index, which again takes O(1) using a HashMap.

=> This results in an overall complexity of O(log n) + O(1) + O(1)

### Remove / Cancel an Order
1. Find the page using index => O(1)
2. Remove order from page's linked hashmap => O(1)
3. Remove order from index => O(1)
=> Overall complexity: O(1) + O(1) + O(1) = O(1)

### Update Order
In case price doesn't change:
1. Find page using index => O(1)
2. Find order in Hashmap => O(1)
3. Adjust amount => O(1)
=> Overall complexity: 3 * O(1) = O(1)

In case price changes:
Cancel + Insert order => O(1) + O(log n) = O(log n)
