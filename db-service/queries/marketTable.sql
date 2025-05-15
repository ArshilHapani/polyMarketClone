-- truncate table polymarket.markets CASCADE;

-- select * from polymarket.markets where status = 'open'::polymarket.market_status;

SELECT 
            o.id, o.user_id, o.market_id,
            o.outcome as "outcome: Outcome",
            o.price, o.quantity, o.filled_quantity,
            o.status as "status: OrderStatus",
            o.side as "side: OrderSide",
            o.created_at, o.updated_at, m.liquidity_b
            FROM polymarket.orders o
            LEFT JOIN polymarket.markets m ON o.market_id = m.id
            WHERE o.status = 'open'::polymarket.order_status 