use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use utility_helpers::log_info;
use uuid::Uuid;

use super::enums::{OrderSide, OrderStatus, Outcome};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Default)]
pub struct Order {
    pub id: Uuid,
    pub user_id: Uuid,
    pub market_id: Uuid,
    pub side: OrderSide,
    pub outcome: Outcome,
    pub price: Decimal,
    pub quantity: Decimal,
    pub filled_quantity: Decimal,
    pub status: OrderStatus,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

// extend order struct with new fields
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct OrderWithMarket {
    pub id: Uuid,
    pub user_id: Uuid,
    pub market_id: Uuid,
    pub side: OrderSide,
    pub outcome: Outcome,
    pub price: Decimal,
    pub quantity: Decimal,
    pub filled_quantity: Decimal,
    pub status: OrderStatus,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub liquidity_b: Decimal,
}

impl From<OrderWithMarket> for Order {
    fn from(order: OrderWithMarket) -> Self {
        Order {
            id: order.id,
            user_id: order.user_id,
            market_id: order.market_id,
            side: order.side,
            outcome: order.outcome,
            price: order.price,
            quantity: order.quantity,
            filled_quantity: order.filled_quantity,
            status: order.status,
            created_at: order.created_at,
            updated_at: order.updated_at,
        }
    }
}

impl Order {
    pub async fn create_order(
        user_id: Uuid,
        market_id: Uuid,
        price: Decimal,
        quantity: Decimal,
        side: OrderSide,
        outcome_side: Outcome,
        pool: &PgPool,
    ) -> Result<Order, sqlx::Error> {
        let order = sqlx::query_as!(
            Order,
            r#"
            INSERT INTO "polymarket"."orders"
            (user_id, market_id, price, quantity, side, outcome)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING 
            id, user_id, market_id,
            outcome as "outcome: Outcome",
            price, quantity, filled_quantity,
            status as "status: OrderStatus",
            side as "side: OrderSide",            
            created_at, updated_at
            "#,
            user_id,
            market_id,
            price,
            quantity,
            side as _,
            outcome_side as _
        )
        .fetch_one(pool)
        .await?;

        log_info!("Order created - {:?}", order.id);
        Ok(order)
    }

    pub async fn delete_order_by_id(order_id: Uuid, pool: &PgPool) -> Result<Order, sqlx::Error> {
        let order = sqlx::query_as!(
            Order,
            r#"
            DELETE FROM polymarket.orders
            WHERE id = $1
            RETURNING 
            id, user_id, market_id,
            outcome as "outcome: Outcome",
            price, quantity, filled_quantity,
            status as "status: OrderStatus",
            side as "side: OrderSide",
            created_at, updated_at
            "#,
            order_id
        )
        .fetch_one(pool)
        .await?;

        log_info!("Order deleted - {:?}", order.id);
        Ok(order)
    }

    pub async fn update_order_status(
        order_id: Uuid,
        status: OrderStatus,
        pool: &PgPool,
    ) -> Result<Order, sqlx::Error> {
        let order = sqlx::query_as!(
            Order,
            r#"
            UPDATE polymarket.orders
            SET status = $1
            WHERE id = $2
            RETURNING 
            id, user_id, market_id,
            outcome as "outcome: Outcome",
            price, quantity, filled_quantity,
            status as "status: OrderStatus",
            side as "side: OrderSide",
            created_at, updated_at
            "#,
            status as _,
            order_id
        )
        .fetch_one(pool)
        .await?;

        log_info!("Order updated - {:?}", order.id);
        Ok(order)
    }

    pub async fn find_order_by_id(order_id: Uuid, pool: &PgPool) -> Result<Order, sqlx::Error> {
        let order = sqlx::query_as!(
            Order,
            r#"
            SELECT 
            id, user_id, market_id,
            outcome as "outcome: Outcome",
            price, quantity, filled_quantity,
            status as "status: OrderStatus",
            side as "side: OrderSide",
            created_at, updated_at            
            FROM polymarket.orders
            WHERE id = $1
            "#,
            order_id
        )
        .fetch_one(pool)
        .await?;

        log_info!("Order found - {:?}", order.id);
        Ok(order)
    }

    pub async fn find_order_by_id_with_market(
        order_id: Uuid,
        pool: &PgPool,
    ) -> Result<OrderWithMarket, sqlx::Error> {
        let order = sqlx::query_as!(
            OrderWithMarket,
            r#"
            SELECT 
            o.id, o.user_id, o.market_id,
            o.outcome as "outcome: Outcome",
            o.price, o.quantity, o.filled_quantity,
            o.status as "status: OrderStatus",
            o.side as "side: OrderSide",
            o.created_at, o.updated_at, m.liquidity_b
            FROM polymarket.orders o
            LEFT JOIN polymarket.markets m ON o.market_id = m.id
            WHERE o.id = $1
            "#,
            order_id
        )
        .fetch_one(pool)
        .await?;

        log_info!("Order found - {:?}", order.id);
        Ok(order)
    }

    pub async fn get_all_open_orders(pool: &PgPool) -> Result<Vec<OrderWithMarket>, sqlx::Error> {
        let orders = sqlx::query_as!(
            OrderWithMarket,
            r#"
            SELECT 
            o.id, o.user_id, o.market_id,
            o.outcome as "outcome: Outcome",
            o.price, o.quantity, o.filled_quantity,
            o.status as "status: OrderStatus",
            o.side as "side: OrderSide",
            o.created_at, o.updated_at, m.liquidity_b
            FROM polymarket.orders o
            JOIN polymarket.markets m ON o.market_id = m.id
            WHERE o.status = 'open'::polymarket.order_status         
            "#,
        )
        .fetch_all(pool)
        .await?;

        Ok(orders)
    }

    pub async fn update(&self, pool: &PgPool) -> Result<Order, sqlx::Error> {
        let order = sqlx::query_as!(
            Order,
            r#"
            UPDATE "polymarket"."orders"
            SET 
                user_id = $1,
                market_id = $2,
                side = $3,
                outcome = $4,
                price = $5,
                quantity = $6,
                filled_quantity = $7,
                status = $8
            WHERE id = $9
            RETURNING 
            id, user_id, market_id,
            outcome as "outcome: Outcome",
            price, quantity, filled_quantity,
            status as "status: OrderStatus",
            side as "side: OrderSide",
            created_at, updated_at
            "#,
            self.user_id,
            self.market_id,
            self.side as _,
            self.outcome as _,
            self.price,
            self.quantity,
            self.filled_quantity,
            self.status as _,
            self.id
        )
        .fetch_one(pool)
        .await?;

        log_info!("Order updated - {:?}", order.id);
        Ok(order)
    }

    pub async fn get_buyer_and_seller_user_id(
        pg_pool: &sqlx::PgPool,
        buy_order_id: Uuid,
        sell_order_id: Uuid,
    ) -> Result<(Uuid, Uuid), sqlx::Error> {
        let order = sqlx::query!(
            r#"
            SELECT user_id FROM polymarket.orders
            WHERE id = $1 OR id = $2
            "#,
            buy_order_id,
            sell_order_id
        )
        .fetch_all(pg_pool)
        .await?;

        if order.len() != 2 {
            return Err(sqlx::Error::RowNotFound);
        }

        Ok((order[0].user_id, order[1].user_id))
    }

    pub async fn update_order_status_and_filled_quantity(
        pool: &PgPool,
        order_id: Uuid,
        order_status: OrderStatus,
        new_filled_quantity: Decimal,
    ) -> Result<Order, sqlx::Error> {
        let order = sqlx::query_as!(
            Order,
            r#"
            UPDATE polymarket.orders
            SET status = $1, filled_quantity = $2
            WHERE id = $3
            RETURNING 
            id, user_id, market_id,
            outcome as "outcome: Outcome",
            price, quantity, filled_quantity,
            status as "status: OrderStatus",
            side as "side: OrderSide",
            created_at, updated_at
            "#,
            order_status as _,
            new_filled_quantity,
            order_id
        )
        .fetch_one(pool)
        .await?;

        log_info!("Order updated - {:?}", order.id);
        Ok(order)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use utility_helpers::types::GoogleClaims;

    use super::*;
    use crate::schema::{market::Market, users::User};

    #[tokio::test]
    // #[ignore = "just like this"]
    async fn test_create_order() {
        dotenv::dotenv().ok();
        let pool = PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
            .await
            .unwrap();

        let user = User::create_new_user(
            &pool,
            &GoogleClaims {
                email: "temp@gmail.com".to_string(),
                exp: 0,
                name: "temp".to_string(),
                picture: "temp".to_string(),
                sub: "temp".to_string(),
            },
        )
        .await
        .unwrap();

        let market = Market::create_new_market(
            "Arshil".to_string(),
            "...".to_string(),
            "...".to_string(),
            Decimal::from_str("10.0").unwrap(),
            &pool,
        )
        .await
        .unwrap();

        // values are taken from the database
        let user_id = user.id;
        let market_id = market.id;

        let price = Decimal::from_str("0.5").unwrap();
        let quantity = Decimal::from_str("1.0").unwrap();
        let side = OrderSide::BUY;

        let order = Order::create_order(
            user_id,
            market_id,
            price,
            quantity,
            side.clone(),
            Outcome::YES,
            &pool,
        )
        .await
        .unwrap();

        assert_eq!(order.user_id, user_id);
        assert_eq!(order.market_id, market_id);
        assert_eq!(order.price, price);
        assert_eq!(order.quantity, quantity);
        assert_eq!(order.side, side);
        assert_eq!(order.filled_quantity, Decimal::ZERO);
        assert_eq!(order.status, OrderStatus::UNSPECIFIED);
        assert_eq!(order.outcome, Outcome::YES);
        assert_eq!(order.created_at, order.updated_at);

        // Clean up
        sqlx::query!(
            r#"
            DELETE FROM "polymarket"."orders"
            WHERE id = $1
            "#,
            order.id
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query!(
            r#"
            DELETE FROM "polymarket"."markets"
            WHERE id = $1
            "#,
            market.id
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query!(
            r#"
            DELETE FROM "polymarket"."users"
            WHERE id = $1
            "#,
            user.id
        )
        .execute(&pool)
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_update_order_status_filled_quantity() {
        dotenv::dotenv().ok();
        let pool = PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
            .await
            .unwrap();

        let user = User::create_new_user(
            &pool,
            &GoogleClaims {
                email: "nami".to_string(),
                exp: 0,
                name: "nami".to_string(),
                picture: "nami".to_string(),
                sub: "nami".to_string(),
            },
        )
        .await
        .unwrap();

        let market = Market::create_new_market(
            "Arshil".to_string(),
            "...".to_string(),
            "...".to_string(),
            Decimal::from_str("10.0").unwrap(),
            &pool,
        )
        .await
        .unwrap();
        // values are taken from the database
        let user_id = user.id;
        let market_id = market.id;
        let price = Decimal::from_str("0.5").unwrap();
        let quantity = Decimal::from_str("1.0").unwrap();
        let side = OrderSide::BUY;
        let order = Order::create_order(
            user_id,
            market_id,
            price,
            quantity,
            side.clone(),
            Outcome::YES,
            &pool,
        )
        .await
        .unwrap();

        assert_eq!(order.user_id, user_id);
        assert_eq!(order.market_id, market_id);
        assert_eq!(order.price, price);
        assert_eq!(order.quantity, quantity);
        assert_eq!(order.side, side);
        assert_eq!(order.filled_quantity, Decimal::ZERO);
        assert_eq!(order.status, OrderStatus::UNSPECIFIED);
        assert_eq!(order.outcome, Outcome::YES);

        // Update the order status to FILLED and set filled quantity
        let new_filled_quantity = Decimal::from_str("1.0").unwrap();
        let updated_order = Order::update_order_status_and_filled_quantity(
            &pool,
            order.id,
            OrderStatus::FILLED,
            new_filled_quantity,
        )
        .await
        .unwrap();

        assert_eq!(updated_order.id, order.id);
        assert_eq!(updated_order.status, OrderStatus::FILLED);
        assert_eq!(updated_order.filled_quantity, new_filled_quantity);

        // Clean up
        sqlx::query!(
            r#"
            DELETE FROM "polymarket"."orders"
            WHERE id = $1
            "#,
            updated_order.id
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query!(
            r#"
            DELETE FROM "polymarket"."markets"
            WHERE id = $1
            "#,
            market.id
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query!(
            r#"
            DELETE FROM "polymarket"."users"
            WHERE id = $1
            "#,
            user.id
        )
        .execute(&pool)
        .await
        .unwrap();

        log_info!("Order updated - {:?}", updated_order.id);
    }
}
