-- Add migration script here

-- enums
CREATE TYPE "polymarket"."market_status" AS ENUM ('OPEN', 'CLOSED', 'SETTLED');
CREATE TYPE "polymarket"."outcome" AS ENUM ('YES', 'NO', 'NO_OUTCOME');
CREATE TYPE "polymarket"."order_side" AS ENUM ('BUY', 'SELL');
CREATE TYPE "polymarket"."order_status" AS ENUM ('OPEN', 'FILLED', 'CANCELLED');
CREATE TYPE "polymarket"."user_transaction_type" AS ENUM ('DEPOSIT', 'WITHDRAWAL', 'TRADE');
CREATE TYPE "polymarket"."user_transaction_status" AS ENUM ('PENDING', 'COMPLETED', 'FAILED');


-- users
CREATE TABLE IF NOT EXISTS "polymarket"."users" (
    "id" uuid DEFAULT gen_random_uuid() PRIMARY KEY,
    -- oAuth2 fields
    "google_id" varchar(255) UNIQUE,
    "email" varchar(255) UNIQUE NOT NULL,
    "name" varchar(255) NOT NULL,
    "avatar" varchar(255) NOT NULL,
    "last_login" timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "refresh_token" TEXT NOT NULL DEFAULT '',

    -- wallet fields
    "public_key" varchar(255) NOT NULL,
    "private_key" TEXT NOT NULL,
    "balance" decimal(20,8) NOT NULL DEFAULT 0,            
    "created_at" timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- markets
CREATE TABLE IF NOT EXISTS "polymarket"."markets" (
    "id" uuid DEFAULT gen_random_uuid() PRIMARY KEY,
    "name" varchar(255) NOT NULL,
    "description" text NOT NULL,
    "logo" varchar(255) NOT NULL,
    "status" "polymarket"."market_status" NOT NULL DEFAULT 'OPEN',
    "liquidity_b" decimal NOT NULL DEFAULT 0,
    "final_outcome" "polymarket"."outcome" NOT NULL DEFAULT 'NO_OUTCOME',
    "created_at" timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- orders
CREATE TABLE IF NOT EXISTS "polymarket"."orders" (
    "id" uuid DEFAULT gen_random_uuid() PRIMARY KEY,
    "user_id" uuid NOT NULL REFERENCES "polymarket"."users"("id"),
    "market_id" uuid NOT NULL REFERENCES "polymarket"."markets"("id"),
    "side" "polymarket"."order_side" NOT NULL,
    "outcome" "polymarket"."outcome" NOT NULL,
    "price" decimal NOT NULL CHECK ("price" >= 0 AND "price" <= 1),
    "quantity" decimal NOT NULL CHECK ("quantity" > 0),
    "filled_quantity" decimal NOT NULL DEFAULT 0,
    "status" "polymarket"."order_status" NOT NULL DEFAULT 'OPEN',
    "created_at" timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- user_trades
CREATE TABLE IF NOT EXISTS "polymarket"."user_trades" (
    "id" uuid DEFAULT gen_random_uuid() PRIMARY KEY,
    "buy_order_id" uuid NOT NULL REFERENCES "polymarket"."orders"("id"),
    "sell_order_id" uuid NOT NULL REFERENCES "polymarket"."orders"("id"),
    "market_id" uuid NOT NULL REFERENCES "polymarket"."markets"("id"),
    "outcome" "polymarket"."outcome" NOT NULL,
    "price" decimal NOT NULL,
    "quantity" decimal NOT NULL CHECK ("quantity" > 0),
    "timestamp" timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "created_at" timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- user_holdings
CREATE TABLE IF NOT EXISTS "polymarket"."user_holdings" (
    "id" uuid DEFAULT gen_random_uuid() PRIMARY KEY,
    "user_id" uuid NOT NULL REFERENCES "polymarket"."users"("id"),
    "market_id" uuid NOT NULL REFERENCES "polymarket"."markets"("id"),    
    "outcome" "polymarket"."outcome" NOT NULL,
    "shares" decimal NOT NULL CHECK ("shares" >= 0),
    "created_at" timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- user_transactions
CREATE TABLE IF NOT EXISTS "polymarket"."user_transactions" (
    "id" uuid DEFAULT gen_random_uuid() PRIMARY KEY,
    "user_id" uuid NOT NULL REFERENCES "polymarket"."users"("id"),
    "amount" decimal NOT NULL CHECK ("amount" > 0),
    "transaction_type" "polymarket"."user_transaction_type" NOT NULL,
    "transaction_status" "polymarket"."user_transaction_status" NOT NULL,
    "tx_hash" varchar(255) NOT NULL,
    "confirmed_at" timestamp,
    "created_at" timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP
);