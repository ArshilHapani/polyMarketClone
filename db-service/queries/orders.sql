-- truncate table polymarket.orders CASCADE;

select * from polymarket.orders where status = 'open'::polymarket.order_status ORDER BY created_at DESC;

-- select * from polymarket.orders ORDER BY created_at DESC;

-- DELETE FROM polymarket.orders
-- WHERE status != ('open'::polymarket.order_status);

 -- SELECT 
 --            o.id, o.user_id, o.market_id,
 --            o.outcome as "outcome: Outcome",
 --            o.price, o.quantity, o.filled_quantity,
 --            o.status as "status: OrderStatus",
 --            o.side as "side: OrderSide",
 --            o.created_at, o.updated_at, m.liquidity_b
 --            FROM polymarket.orders o
 --            JOIN polymarket.markets m ON o.market_id = m.id
 --            WHERE o.status = 'open'::polymarket.order_status;

 -- select * from polymarket.orders;