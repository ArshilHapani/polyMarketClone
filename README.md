![Architecture v2](./assets/architecture_v2_1.png)

## Important rpk commands

- `rpk topic consume price-updates -n 10` - Consume the last 10 messages from the `price-updates` topic.
- `rpk group seek consumer-group-price-updates --topics price-updates --to=start --allow-new-topics` - Seek the consumer group `consumer-group-price-updates` to the start of the `price-updates` topic.

# Used magic

- Redpanda - A high-performance streaming platform.
- ClickHouse - A fast open-source OLAP database management system.
- Bloom filters - A space-efficient probabilistic data structure to test whether an element is a member of a set.
- NATS - A high performance message queues

# Notes

NATS can produce multiple messages parallelly, it have multiple streams, each stream acts as separate queue, so we have to use different streams for different services
