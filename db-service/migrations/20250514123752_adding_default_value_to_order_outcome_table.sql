-- Add migration script here

ALTER TABLE polymarket.orders
ALTER COLUMN outcome SET DEFAULT 'unspecified';

UPDATE polymarket.orders
SET outcome = 'unspecified'
WHERE outcome IS NULL;