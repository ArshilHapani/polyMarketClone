import { Box, Button, Flex, NumberInput, Text } from "@chakra-ui/react";
import { useState } from "react";

import { toaster } from "@/components/ui/toaster";

type Props = {
  mode: "buy" | "sell";
  stockMode: "yes" | "no";
};

const MarketOrderForm = ({ mode }: Props) => {
  const [amount, setAmount] = useState("");

  function handleSubmit() {
    console.log({ amount });
    if (amount === "") {
      toaster.error({
        title: "Amount is required",
      });
      return;
    }
    toaster.success({
      title: "TODO",
    });
  }

  return (
    <Box>
      <Flex mt={4}>
        <Text fontSize="lg" color="gray.600" fontWeight="semibold" width="1/3">
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
      <Flex mt={3} justifyContent="end">
        <Flex gap={2} alignItems="center">
          {PREDEFINED_AMOUNTS.map((amount) => (
            <Button
              key={amount}
              variant="outline"
              fontSize="xs"
              rounded="full"
              bg="transparent"
              border="1px solid"
              borderColor="gray.300"
              paddingX={5}
              size="xs"
              onClick={() =>
                setAmount((prev) => (Number(prev) + Number(amount)).toString())
              }
            >
              ${amount}
            </Button>
          ))}
        </Flex>
      </Flex>

      <Button
        width="full"
        mt={4}
        bg="blue.600/90"
        _hover={{ bg: "blue.600" }}
        onClick={handleSubmit}
      >
        {mode === "buy" ? "Buy" : "Sell"} Now
      </Button>
    </Box>
  );
};

export default MarketOrderForm;
const PREDEFINED_AMOUNTS = [1, 20, 100];
