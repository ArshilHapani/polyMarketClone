-- truncate table polymarket.user_trades;

select * from polymarket.user_trades order by created_at DESC;

-- SELECT
--     market_id,
--     SUM(quantity) AS total_volume
-- FROM polymarket.user_trades
-- WHERE timestamp >= NOW() - INTERVAL '6 hours'
-- GROUP BY market_id;