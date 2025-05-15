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
    pub liquidity_b: Option<Decimal>,
}

impl From<&OrderWithMarket> for Order {
    fn from(order: &OrderWithMarket) -> Self {
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
        pool: &PgPool,
    ) -> Result<Order, sqlx::Error> {
        let order = sqlx::query_as!(
            Order,
            r#"
            INSERT INTO "polymarket"."orders"
            (user_id, market_id, price, quantity, side)
            VALUES ($1, $2, $3, $4, $5)
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
            DELETE FROM "polymarket"."orders"
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
            UPDATE "polymarket"."orders"
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
            FROM "polymarket"."orders"
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
            LEFT JOIN polymarket.markets m ON o.market_id = m.id
            WHERE o.status = 'open'::polymarket.order_status            
            "#,
        )
        .fetch_all(pool)
        .await?;

        log_info!("Orders found - {:?}", orders.len());
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
}

impl OrderWithMarket {
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
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use utility_helpers::types::GoogleClaims;

    use super::*;
    use crate::schema::{market::Market, users::User};

    #[tokio::test]
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

        let order = Order::create_order(user_id, market_id, price, quantity, side.clone(), &pool)
            .await
            .unwrap();

        assert_eq!(order.user_id, user_id);
        assert_eq!(order.market_id, market_id);
        assert_eq!(order.price, price);
        assert_eq!(order.quantity, quantity);
        assert_eq!(order.side, side);
        assert_eq!(order.filled_quantity, Decimal::ZERO);
        assert_eq!(order.status, OrderStatus::OPEN);
        assert_eq!(order.outcome, Outcome::UNSPECIFIED);
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
}
