import axios from "axios";
import jsCookies from "js-cookie";

import { marketServiceClient, priceServiceClient } from "../grpc/clients";
import { GetUserOrdersPaginatedResponse, GetUserResponse } from "../types/api";
import { OrderCategory } from "../types";
import { Timeframe } from "@/generated/grpc_service_types/common";

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

  static async getOrderBook(marketId: string, depth: number = 10) {
    try {
      const { response } = await marketServiceClient.getMarketBook({
        depth,
        marketId,
      });
      return response;
    } catch (error: any) {
      console.error("Failed to get order book: ", error);
      return null;
    }
  }

  static async getTopTenHolders(marketId: string) {
    try {
      const { response } = await marketServiceClient.getTopHolders({
        marketId,
      });
      return response.topHolders;
    } catch (error: any) {
      console.error("Failed to get top ten holders: ", error);
      return [];
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
    orderType: OrderCategory = "all",
  ) {
    try {
      const { data } = await axios.get<GetUserOrdersPaginatedResponse>(
        `${BASE_URL}/user/orders/get/${marketId}?page=${page}&page_size=${pageSize}&status=${orderType}`,
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
      return {
        orders: [],
        page: 0,
        page_size: 0,
        holdings: { no: "0", yes: "0" },
      };
    }
  }
}

export class ChartGetters {
  static async getChartDataWithinTimeRange(
    marketId: string,
    timeframe: Timeframe,
  ) {
    try {
      const { response } = await priceServiceClient.getPriceDataWithinInterval({
        marketId,
        timeframe,
      });
      return response;
    } catch (error: any) {
      console.error("Failed to get chart data: ", error);
      return {
        marketId: "",
        priceData: [],
      };
    }
  }
}
