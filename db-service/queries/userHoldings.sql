-- truncate table polymarket.user_holdings;


-- INSERT INTO polymarket.user_holdings (user_id, market_id, shares)
--             VALUES ('24fa20ac-822f-49e9-9cb6-e25e940ad608'::uuid, 'bd609b17-d3d3-4f70-a5e2-0a3f3aa2160c'::uuid, -10)
--             ON CONFLICT (user_id, market_id)
-- 			DO UPDATE SET shares = polymarket.user_holdings.shares + -10,
--             updated_at = NOW()
--             RETURNING id, user_id, market_id, shares, created_at, updated_at;
			

-- INSERT INTO polymarket.user_holdings (user_id, market_id, shares)
--             VALUES ('24fa20ac-822f-49e9-9cb6-e25e940ad608'::uuid, 'bd609b17-d3d3-4f70-a5e2-0a3f3aa2160c'::uuid, 200)
--             ON CONFLICT (user_id, market_id) DO NOTHING;

-- select * from polymarket.user_holdings order by created_at DESC;

select 
	uh.user_id,
	uh.shares,
	u.balance,
	u.email
FROM 
	polymarket.user_holdings uh
JOIN
	polymarket.users u ON uh.user_id = u.id
ORDER BY
	uh.created_at DESC;