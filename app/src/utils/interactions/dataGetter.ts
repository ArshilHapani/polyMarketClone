import axios from "axios";

import { GetPaginatedMarketResponse } from "@/generated/markets";
import { decodeProtoMessage } from "../protoHelpers";
import { axiosInstance } from "./axios";

export class MarketGetters {
  static async getMarketData(page: number, pageSize: number) {
    try {
      const { data, status } = await axiosInstance.get(
        `/market/getAll?page=${page}&pageSize=${pageSize}`,
        {
          headers: {
            "Content-Type": "application/x-protobuf",
          },
          responseType: "arraybuffer",
        }
      );

      if (status != 200) {
        throw new Error("Failed to fetch market data");
      }
      console.log({ data });
      const decodedData = await decodeProtoMessage<GetPaginatedMarketResponse>(
        "/proto/markets.proto",
        "markets.GetPaginatedMarketResponse",
        data
      );

      return decodedData;
    } catch (error: any) {
      console.error("Error fetching market data:", error);
      return [];
    }
  }
}
