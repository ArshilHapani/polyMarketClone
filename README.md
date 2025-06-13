![Architecture v2](./assets/architecture_v2.png)

## Important rpk commands

- `rpk topic consume price-updates -n 10` - Consume the last 10 messages from the `price-updates` topic.
- `rpk group seek consumer-group-price-updates --topics price-updates --to=start --allow-new-topics` - Seek the consumer group `consumer-group-price-updates` to the start of the `price-updates` topic.
