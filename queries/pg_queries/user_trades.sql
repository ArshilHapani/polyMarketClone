-- truncate table polymarket.user_trades;

-- select * from polymarket.user_trades order by created_at DESC;

-- SELECT
--     market_id,
--     SUM(quantity) AS total_volume
-- FROM polymarket.user_trades
-- WHERE timestamp >= NOW() - INTERVAL '6 hours'
-- GROUP BY market_id;

select t.id ,u.name, u.email, u.avatar, t.trade_type, t.outcome, t.price, t.quantity, t.timestamp
FROM polymarket.user_trades t
RIGHT JOIN polymarket.users u ON u.id = t.user_id
WHERE u.name != 'Admin' AND t.market_id = '20ec3758-04ef-4300-a24c-c9019cf55c95'::uuid
ORDER BY t.timestamp DESC;