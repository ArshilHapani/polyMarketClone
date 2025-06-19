import { marketServiceClient } from "../grpc/clients";

export class MarketGetters {
  static async getMarketData(page: number, pageSize: number) {
    try {
      const data = await marketServiceClient.getMarketData({
        page,
        pageSize,
      });
      console.log("Market data fetched successfully:", data);
      return data.response.markets;
    } catch (error: any) {
      console.error("Error fetching market data:", error);
      return [];
    }
  }
}
