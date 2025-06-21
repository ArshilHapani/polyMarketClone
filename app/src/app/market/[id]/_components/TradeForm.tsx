"use client";

import { Box, Button, Flex, NumberInput, Text } from "@chakra-ui/react";
import { useState } from "react";

type Props = {
  mode: "buy" | "sell";
};

const TradeForm = ({ mode }: Props) => {
  const [amount, setAmount] = useState("");
  return (
    <Box>
      <Flex gap={2} width="100%" justifyContent="space-between">
        <Button width="1/2" bg="green.600/90" _hover={{ bg: "green.600" }}>
          Yes $5
        </Button>
        <Button width="1/2" bg="red.600/90" _hover={{ bg: "red.600" }}>
          No $5
        </Button>
      </Flex>

      {/* market order field */}
      <Box>
        <Flex mt={4}>
          <Text
            fontSize="lg"
            color="gray.600"
            fontWeight="semibold"
            width="1/3"
          >
            Amount
          </Text>
          <NumberInput.Root
            formatOptions={{
              style: "currency",
              currency: "USD",
              currencyDisplay: "symbol",
              currencySign: "accounting",
            }}
          >
            <NumberInput.Input
              width="full"
              dir="rtl"
              outline="none"
              border="none"
              placeholder="$10"
              fontSize="4xl"
              fontWeight="extrabold"
              value={amount}
              onChange={(e) => setAmount(e.target.value)}
            />
          </NumberInput.Root>
        </Flex>
        {/* pre defined amount setter */}
        <Flex mt={3} gap={2} justifyContent={"end"}>
          {PREDEFINED_AMOUNTS.map((amount) => (
            <Button
              key={amount}
              variant="outline"
              rounded="full"
              bg="transparent"
              border="1px solid"
              borderColor="gray.300"
              padding={1}
            >
              ${amount}
            </Button>
          ))}
        </Flex>

        <Button
          type="submit"
          width="full"
          mt={4}
          bg="blue.600/90"
          _hover={{ bg: "blue.600" }}
        >
          {mode === "buy" ? "Buy" : "Sell"} Now
        </Button>
      </Box>
    </Box>
  );
};

export default TradeForm;

const PREDEFINED_AMOUNTS = [1, 20, 100];
