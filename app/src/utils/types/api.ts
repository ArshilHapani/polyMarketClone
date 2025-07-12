import { Order } from ".";

export interface BaseResponse {
  message: string;
  success: boolean;
}
export interface ErrorResponse {
  error: string;
}

export interface LoginResponse extends BaseResponse {
  userId: string;
  sessionToken: string;
}

export interface GetUserResponse {
  avatar: string;
  balance: number;
  email: string;
  name: string;
  public_key: string;
}

export interface GetUserOrdersPaginatedResponse {
  orders: Order[];
  page: number;
  page_size: number;
  total_pages: number;
  holdings: {
    no: string;
    yes: string;
  };
}

export interface GetUserMetadataResponse {
  profile_insight: {
    avatar: string;
    avg_fill_ratio: string;
    avg_trade_price: string;
    balance: string;
    created_at: string;
    email: string;
    first_trade_at: string;
    id: string;
    last_deposit: null;
    last_login: string;
    last_trade_at: string;
    last_withdraw: null;
    markets_traded: number;
    max_trade_qty: string;
    name: string;
    open_orders: number;
    partial_orders: number;
    public_key: string;
    total_deposit: null;
    total_orders: number;
    total_trades: number;
    total_volume: string;
    total_withdraw: null;
  };
  user_id: string;
}
