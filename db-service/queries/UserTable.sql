-- truncate table polymarket.users cascade;

SELECT id, public_key, private_key, balance, created_at, updated_at
	FROM polymarket.users;