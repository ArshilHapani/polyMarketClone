-- truncate table polymarket.user_holdings;


-- INSERT INTO polymarket.user_holdings (user_id, market_id, shares)
--             VALUES ('24fa20ac-822f-49e9-9cb6-e25e940ad608'::uuid, 'bd609b17-d3d3-4f70-a5e2-0a3f3aa2160c'::uuid, -10)
--             ON CONFLICT (user_id, market_id)
-- 			DO UPDATE SET shares = polymarket.user_holdings.shares + -10,
--             updated_at = NOW()
--             RETURNING id, user_id, market_id, shares, created_at, updated_at;
			
select * from polymarket.user_holdings;