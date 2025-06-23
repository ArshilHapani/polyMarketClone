export type Order = {
  created_at: string;
  filled_quantity: string;
  id: string;
  market_id: string;
  outcome: "YES" | "NO" | "SETTLED";
  price: string;
  quantity: string;
  side: "BUY" | "SELL";
  status: "OPEN" | "CLOSE" | "UNSPECIFIED";
  updated_at: string;
  user_id: string;
};
