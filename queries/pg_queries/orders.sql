-- truncate table polymarket.orders CASCADE;

-- select count(*) from polymarket.orders where status = 'open'::polymarket.order_status;
-- select * from polymarket.orders where id = 'c6df1d26-e223-4dc3-98a9-663eb51b293f'::uuid ORDER BY created_at DESC;

-- select price, status from polymarket.orders where status in ('open'::polymarket.order_status, 'pending_update'::polymarket.order_status) group by price, status;

select * from polymarket.orders
	where id = '4bcb302a-ab5f-476b-9593-c4a4bc3e06bc'
order by created_at DESC;

-- select * from polymarket.orders where status = 'open'::polymarket.order_status ORDER BY created_at DESC;


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