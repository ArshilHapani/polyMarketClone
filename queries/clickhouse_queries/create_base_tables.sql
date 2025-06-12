-- db is already created

USE polyMarket;

-- Core table
CREATE TABLE market_price_data (
    market_id UUID,
    yes_price Decimal(20, 8),
    no_price Decimal(20, 8),
    ts DateTime,
) ENGINE = MergeTree
ORDER BY ts;


-- kafka engine table
CREATE TABLE market_price_data_kafka (
    market_id UUID,
    yes_price Decimal(20, 8),
    no_price Decimal(20, 8),
    ts DateTime,   
) ENGINE = Kafka(
    'polyMarket_redpanda:9092', -- broker (red panda)
    'price-updates', -- topic
    'consumer-group-price-updates', -- consumer group
    'JSONEachRow' -- format
);

-- materialized view to copy data from kafka to core table
DROP TABLE IF EXISTS market_price_data_mv;
CREATE MATERIALIZED VIEW market_price_data_mv
TO market_price_data AS
SELECT 
    market_id,
    yes_price AS price_yes,
    no_price AS price_no,
    toDateTime(ts) AS ts
FROM market_price_data_kafka;