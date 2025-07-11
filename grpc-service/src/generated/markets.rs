// This file is @generated by prost-build.
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Clone, Copy, PartialEq, ::prost::Message)]
pub struct GetMarketDataRequest {
    /// Pagination request
    #[prost(message, optional, tag = "1")]
    pub page_request: ::core::option::Option<super::common::PageRequest>,
    /// Status of the market
    #[prost(enumeration = "MarketStatus", tag = "2")]
    pub market_status: i32,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Clone, Copy, PartialEq, ::prost::Message)]
pub struct OrderLevel {
    /// Price of the order
    #[prost(double, tag = "1")]
    pub price: f64,
    /// Total quantity of shares at this price
    #[prost(double, tag = "2")]
    pub shares: f64,
    /// Number of users at this price (histogram)
    #[prost(uint32, tag = "3")]
    pub users: u32,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestWithMarketIdAndPageRequest {
    /// ID of the market
    #[prost(string, tag = "1")]
    pub market_id: ::prost::alloc::string::String,
    /// Pagination request
    #[prost(message, optional, tag = "2")]
    pub page_request: ::core::option::Option<super::common::PageRequest>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MarketTrade {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub avatar: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub email: ::prost::alloc::string::String,
    #[prost(enumeration = "TradeType", tag = "5")]
    pub trade_type: i32,
    #[prost(enumeration = "Outcome", tag = "6")]
    pub outcome: i32,
    #[prost(double, tag = "7")]
    pub price: f64,
    #[prost(double, tag = "8")]
    pub quantity: f64,
    #[prost(string, tag = "9")]
    pub created_at: ::prost::alloc::string::String,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetMarketTradesResponse {
    /// ID of the market
    #[prost(string, tag = "1")]
    pub market_id: ::prost::alloc::string::String,
    /// List of trades for the market
    #[prost(message, repeated, tag = "2")]
    pub trades: ::prost::alloc::vec::Vec<MarketTrade>,
    /// Pagination info
    #[prost(message, optional, tag = "3")]
    pub page_info: ::core::option::Option<super::common::PageInfo>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OrderBook {
    #[prost(message, repeated, tag = "1")]
    pub bids: ::prost::alloc::vec::Vec<OrderLevel>,
    #[prost(message, repeated, tag = "2")]
    pub asks: ::prost::alloc::vec::Vec<OrderLevel>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetMarketBookResponse {
    /// ID of the market
    #[prost(string, tag = "1")]
    pub market_id: ::prost::alloc::string::String,
    /// Order book for YES outcome
    #[prost(message, optional, tag = "2")]
    pub yes_book: ::core::option::Option<OrderBook>,
    /// Order book for NO outcome
    #[prost(message, optional, tag = "3")]
    pub no_book: ::core::option::Option<OrderBook>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestWithMarketId {
    #[prost(string, tag = "1")]
    pub market_id: ::prost::alloc::string::String,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestForMarketBook {
    #[prost(string, tag = "1")]
    pub market_id: ::prost::alloc::string::String,
    #[prost(uint32, tag = "2")]
    pub depth: u32,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Market {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub description: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub logo: ::prost::alloc::string::String,
    #[prost(enumeration = "MarketStatus", tag = "5")]
    pub status: i32,
    #[prost(double, tag = "6")]
    pub liquidity_b: f64,
    #[prost(enumeration = "Outcome", tag = "7")]
    pub final_outcome: i32,
    #[prost(string, tag = "8")]
    pub created_at: ::prost::alloc::string::String,
    #[prost(string, tag = "9")]
    pub updated_at: ::prost::alloc::string::String,
    #[prost(string, tag = "10")]
    pub market_expiry: ::prost::alloc::string::String,
    #[prost(float, tag = "11")]
    pub yes_price: f32,
    #[prost(float, tag = "12")]
    pub no_price: f32,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VolumeInfo {
    #[prost(string, tag = "1")]
    pub market_id: ::prost::alloc::string::String,
    #[prost(double, tag = "2")]
    pub yes_buy_qty: f64,
    #[prost(double, tag = "3")]
    pub yes_buy_usd: f64,
    #[prost(double, tag = "4")]
    pub yes_sell_qty: f64,
    #[prost(double, tag = "5")]
    pub yes_sell_usd: f64,
    #[prost(double, tag = "6")]
    pub no_buy_qty: f64,
    #[prost(double, tag = "7")]
    pub no_buy_usd: f64,
    #[prost(double, tag = "8")]
    pub no_sell_qty: f64,
    #[prost(double, tag = "9")]
    pub no_sell_usd: f64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MarketPrice {
    #[prost(string, tag = "1")]
    pub market_id: ::prost::alloc::string::String,
    #[prost(double, tag = "2")]
    pub latest_yes_price: f64,
    #[prost(double, tag = "3")]
    pub latest_no_price: f64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetMarketByIdResponse {
    #[prost(message, optional, tag = "1")]
    pub market: ::core::option::Option<Market>,
    #[prost(message, optional, tag = "2")]
    pub volume_info: ::core::option::Option<VolumeInfo>,
    #[prost(message, optional, tag = "3")]
    pub market_price: ::core::option::Option<MarketPrice>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UserWithTotalHoldings {
    #[prost(string, tag = "1")]
    pub user_id: ::prost::alloc::string::String,
    #[prost(double, tag = "2")]
    pub total_shares: f64,
    #[prost(double, tag = "3")]
    pub total_yes_shares: f64,
    #[prost(double, tag = "4")]
    pub total_no_shares: f64,
    #[prost(string, tag = "5")]
    pub username: ::prost::alloc::string::String,
    #[prost(string, tag = "6")]
    pub avatar: ::prost::alloc::string::String,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetTopHoldersResponse {
    #[prost(string, tag = "1")]
    pub market_id: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "2")]
    pub top_holders: ::prost::alloc::vec::Vec<UserWithTotalHoldings>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetPaginatedMarketResponse {
    #[prost(message, repeated, tag = "1")]
    pub markets: ::prost::alloc::vec::Vec<Market>,
    #[prost(message, optional, tag = "2")]
    pub page_info: ::core::option::Option<super::common::PageInfo>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum MarketStatus {
    UnspecifiedMarketStatus = 0,
    Open = 1,
    Closed = 2,
    Settled = 3,
}
impl MarketStatus {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::UnspecifiedMarketStatus => "UNSPECIFIED_MARKET_STATUS",
            Self::Open => "OPEN",
            Self::Closed => "CLOSED",
            Self::Settled => "SETTLED",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "UNSPECIFIED_MARKET_STATUS" => Some(Self::UnspecifiedMarketStatus),
            "OPEN" => Some(Self::Open),
            "CLOSED" => Some(Self::Closed),
            "SETTLED" => Some(Self::Settled),
            _ => None,
        }
    }
}
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum Outcome {
    UnspecifiedOutcome = 0,
    Yes = 1,
    No = 2,
    Unspecified = 3,
}
impl Outcome {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::UnspecifiedOutcome => "UNSPECIFIED_OUTCOME",
            Self::Yes => "YES",
            Self::No => "NO",
            Self::Unspecified => "UNSPECIFIED",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "UNSPECIFIED_OUTCOME" => Some(Self::UnspecifiedOutcome),
            "YES" => Some(Self::Yes),
            "NO" => Some(Self::No),
            "UNSPECIFIED" => Some(Self::Unspecified),
            _ => None,
        }
    }
}
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum TradeType {
    UnspecifiedTradeType = 0,
    /// Buy trade
    Buy = 1,
    /// Sell trade
    Sell = 2,
}
impl TradeType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::UnspecifiedTradeType => "UNSPECIFIED_TRADE_TYPE",
            Self::Buy => "BUY",
            Self::Sell => "SELL",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "UNSPECIFIED_TRADE_TYPE" => Some(Self::UnspecifiedTradeType),
            "BUY" => Some(Self::Buy),
            "SELL" => Some(Self::Sell),
            _ => None,
        }
    }
}
/// Generated server implementations.
pub mod market_service_server {
    #![allow(
        unused_variables,
        dead_code,
        missing_docs,
        clippy::wildcard_imports,
        clippy::let_unit_value,
    )]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with MarketServiceServer.
    #[async_trait]
    pub trait MarketService: std::marker::Send + std::marker::Sync + 'static {
        async fn get_market_data(
            &self,
            request: tonic::Request<super::GetMarketDataRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetPaginatedMarketResponse>,
            tonic::Status,
        >;
        async fn get_market_by_id(
            &self,
            request: tonic::Request<super::RequestWithMarketId>,
        ) -> std::result::Result<
            tonic::Response<super::GetMarketByIdResponse>,
            tonic::Status,
        >;
        async fn get_market_book(
            &self,
            request: tonic::Request<super::RequestForMarketBook>,
        ) -> std::result::Result<
            tonic::Response<super::GetMarketBookResponse>,
            tonic::Status,
        >;
        async fn get_top_holders(
            &self,
            request: tonic::Request<super::RequestWithMarketId>,
        ) -> std::result::Result<
            tonic::Response<super::GetTopHoldersResponse>,
            tonic::Status,
        >;
        async fn get_market_trades(
            &self,
            request: tonic::Request<super::RequestWithMarketIdAndPageRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetMarketTradesResponse>,
            tonic::Status,
        >;
    }
    #[derive(Debug)]
    pub struct MarketServiceServer<T> {
        inner: Arc<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    impl<T> MarketServiceServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
                max_decoding_message_size: None,
                max_encoding_message_size: None,
            }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
        /// Enable decompressing requests with the given encoding.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding);
            self
        }
        /// Compress responses with the given encoding, if the client supports it.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.max_decoding_message_size = Some(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.max_encoding_message_size = Some(limit);
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for MarketServiceServer<T>
    where
        T: MarketService,
        B: Body + std::marker::Send + 'static,
        B::Error: Into<StdError> + std::marker::Send + 'static,
    {
        type Response = http::Response<tonic::body::Body>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<std::result::Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            match req.uri().path() {
                "/markets.MarketService/GetMarketData" => {
                    #[allow(non_camel_case_types)]
                    struct GetMarketDataSvc<T: MarketService>(pub Arc<T>);
                    impl<
                        T: MarketService,
                    > tonic::server::UnaryService<super::GetMarketDataRequest>
                    for GetMarketDataSvc<T> {
                        type Response = super::GetPaginatedMarketResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetMarketDataRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as MarketService>::get_market_data(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = GetMarketDataSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/markets.MarketService/GetMarketById" => {
                    #[allow(non_camel_case_types)]
                    struct GetMarketByIdSvc<T: MarketService>(pub Arc<T>);
                    impl<
                        T: MarketService,
                    > tonic::server::UnaryService<super::RequestWithMarketId>
                    for GetMarketByIdSvc<T> {
                        type Response = super::GetMarketByIdResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RequestWithMarketId>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as MarketService>::get_market_by_id(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = GetMarketByIdSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/markets.MarketService/GetMarketBook" => {
                    #[allow(non_camel_case_types)]
                    struct GetMarketBookSvc<T: MarketService>(pub Arc<T>);
                    impl<
                        T: MarketService,
                    > tonic::server::UnaryService<super::RequestForMarketBook>
                    for GetMarketBookSvc<T> {
                        type Response = super::GetMarketBookResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RequestForMarketBook>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as MarketService>::get_market_book(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = GetMarketBookSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/markets.MarketService/GetTopHolders" => {
                    #[allow(non_camel_case_types)]
                    struct GetTopHoldersSvc<T: MarketService>(pub Arc<T>);
                    impl<
                        T: MarketService,
                    > tonic::server::UnaryService<super::RequestWithMarketId>
                    for GetTopHoldersSvc<T> {
                        type Response = super::GetTopHoldersResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RequestWithMarketId>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as MarketService>::get_top_holders(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = GetTopHoldersSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/markets.MarketService/GetMarketTrades" => {
                    #[allow(non_camel_case_types)]
                    struct GetMarketTradesSvc<T: MarketService>(pub Arc<T>);
                    impl<
                        T: MarketService,
                    > tonic::server::UnaryService<
                        super::RequestWithMarketIdAndPageRequest,
                    > for GetMarketTradesSvc<T> {
                        type Response = super::GetMarketTradesResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::RequestWithMarketIdAndPageRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as MarketService>::get_market_trades(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = GetMarketTradesSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        let mut response = http::Response::new(
                            tonic::body::Body::default(),
                        );
                        let headers = response.headers_mut();
                        headers
                            .insert(
                                tonic::Status::GRPC_STATUS,
                                (tonic::Code::Unimplemented as i32).into(),
                            );
                        headers
                            .insert(
                                http::header::CONTENT_TYPE,
                                tonic::metadata::GRPC_CONTENT_TYPE,
                            );
                        Ok(response)
                    })
                }
            }
        }
    }
    impl<T> Clone for MarketServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
                max_decoding_message_size: self.max_decoding_message_size,
                max_encoding_message_size: self.max_encoding_message_size,
            }
        }
    }
    /// Generated gRPC service name
    pub const SERVICE_NAME: &str = "markets.MarketService";
    impl<T> tonic::server::NamedService for MarketServiceServer<T> {
        const NAME: &'static str = SERVICE_NAME;
    }
}
