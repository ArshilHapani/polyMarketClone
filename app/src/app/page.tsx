"use client";

import { Container, Text } from "@chakra-ui/react";
import { useQuery } from "@tanstack/react-query";

import { MarketGetters } from "@/utils/interactions/dataGetter";

export default function Home() {
  const { data, error } = useQuery({
    queryFn: () => MarketGetters.getMarketData(1, 10),
    queryKey: ["marketData", 1, 10],
  });
  return (
    <Container my={10}>
      <Text fontSize="2xl" fontWeight="bold" mb={4}>
        Trending Markets
      </Text>
    </Container>
  );
}
