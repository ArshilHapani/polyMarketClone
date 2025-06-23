"use client";

import { Box, Button, Flex } from "@chakra-ui/react";
import { useState } from "react";

import MarketOrderForm from "./MarketOrderForm";
import LimitOrderForm from "./LimitOrderForm";

type Props = {
  mode: "buy" | "sell";
  orderType: "market" | "limit";
  market_id: string;
};

const TradeForm = ({ mode, orderType, market_id }: Props) => {
  const [stockMode, setStockMode] = useState<"yes" | "no">("yes");

  return (
    <Box>
      <Flex gap={2} width="100%" justifyContent="space-between">
        <Button
          width="1/2"
          bg={stockMode === "yes" ? "green.600" : "gray.500"}
          _hover={{ bg: "green.600" }}
          onClick={() => setStockMode("yes")}
        >
          Yes $5
        </Button>
        <Button
          width="1/2"
          bg={stockMode === "no" ? "red.600" : "gray.500"}
          _hover={{ bg: "red.600" }}
          onClick={() => setStockMode("no")}
        >
          No $5
        </Button>
      </Flex>

      {/* market / limit order form */}
      {orderType === "limit" ? (
        <LimitOrderForm
          mode={mode}
          stockMode={stockMode}
          market_id={market_id}
        />
      ) : (
        <MarketOrderForm mode={mode} stockMode={stockMode} />
      )}
    </Box>
  );
};

export default TradeForm;
