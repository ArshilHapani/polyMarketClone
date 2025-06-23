import axios, { AxiosError } from "axios";
import jsCookies from "js-cookie";

import { LoginResponse } from "../types/api";

const TOKEN = jsCookies.get("polymarketAuthToken") || "";
const BASE_URL = process.env.NEXT_PUBLIC_SERVICE_API_URL || "";

export class UserAuthActions {
  static async handleSignInWithGoogle({ id_token }: { id_token: string }) {
    const { data, status } = await axios.post(`${BASE_URL}/login`, {
      id_token,
    });

    if (status != 200) throw new Error(data.error);
    return data as LoginResponse;
  }
}

export class MarketActions {
  static async createLimitOrder(reqPayload: {
    market_id: string;
    price: number;
    quantity: number;
    side: "buy" | "sell";
    outcome_side: "yes" | "no";
  }) {
    try {
      const { status, data } = await axios.post(
        `${BASE_URL}/user/orders/create`,
        reqPayload,
        {
          headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${TOKEN}`,
          },
        },
      );
    } catch (error: any) {
      console.error("Error creating limit order:", error);
      if (error instanceof AxiosError) {
        throw new Error(
          error.response?.data?.error || "Failed to create limit order",
        );
      }
      throw new Error("Failed to create limit order");
    }
  }
}
