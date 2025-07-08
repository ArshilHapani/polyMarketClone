export type Order = {
  created_at: string;
  filled_quantity: string;
  id: string;
  market_id: string;
  outcome: "yes" | "no" | "settled";
  price: string;
  quantity: string;
  side: "buy" | "sell";
  status: "OPEN" | "CLOSE" | "UNSPECIFIED";
  updated_at: string;
  user_id: string;
};

export type OrderType =
  | "open"
  | "cancelled"
  | "filled"
  | "expired"
  | "pending_update"
  | "pending_cancel";
