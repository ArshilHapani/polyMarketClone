import axios from "axios";
import jsCookies from "js-cookie";

import { marketServiceClient } from "../grpc/clients";
import { GetUserOrdersPaginatedResponse, GetUserResponse } from "../types/api";

const TOKEN = jsCookies.get("polymarketAuthToken") || "";
const BASE_URL = process.env.NEXT_PUBLIC_SERVICE_API_URL || "";

export class MarketGetters {
  static async getMarketData(page: number, pageSize: number) {
    try {
      const data = await marketServiceClient.getMarketData({
        page,
        pageSize,
      });
      return data.response.markets;
    } catch (error: any) {
      console.error("Error fetching market data:", error);
      return [];
    }
  }

  static async getMarketById(marketId: string) {
    try {
      const { response } = await marketServiceClient.getMarketById({
        marketId,
      });
      return response;
    } catch (error: any) {
      console.log("Failed to get market due to ", error);
      return null;
    }
  }
}

export class UserGetters {
  static async getUserData() {
    try {
      const { data, status } = await axios.get<GetUserResponse>(
        `${BASE_URL}/user/profile`,
        {
          headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${TOKEN}`,
          },
        },
      );
      if (status !== 200) {
        throw new Error("Failed to fetch user data");
      }
      return data;
    } catch (e: any) {
      console.log("Error fetching user data:", e);
      return null;
    }
  }
}

export class OrderGetters {
  static async getUserOrdersPaginated(page: number, pageSize: number) {
    try {
      const { data } = await axios.get<GetUserOrdersPaginatedResponse>(
        `${BASE_URL}/user/orders/get?page=${page}&page_size=${pageSize}`,
        {
          headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${TOKEN}`,
          },
        },
      );

      return data;
    } catch (error: any) {
      console.error("Failed to get orders ", error);
      return { orders: [], page: 0, page_size: 0 };
    }
  }

  static async getUserOrdersByMarket(
    marketId: string,
    page: number,
    pageSize: number,
  ) {
    try {
      const { data } = await axios.get<GetUserOrdersPaginatedResponse>(
        `${BASE_URL}/user/orders/get/${marketId}?page=${page}&page_size=${pageSize}`,
        {
          headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${TOKEN}`,
          },
        },
      );

      return data;
    } catch (error: any) {
      console.error("Failed to get orders ", error);
      return { orders: [], page: 0, page_size: 0 };
    }
  }
}
