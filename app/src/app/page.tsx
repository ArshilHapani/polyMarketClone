"use client";

import { Container, HStack, Text } from "@chakra-ui/react";
import { useQuery } from "@tanstack/react-query";

import { MarketGetters } from "@/utils/interactions/dataGetter";
import TrendingMarketCard from "@/components/TrendingMarketCard";

export default function Home() {
  const { data, isLoading } = useQuery({
    queryFn: () => MarketGetters.getMarketData(1, 10),
    queryKey: ["marketData", 1, 10],
  });
  return (
    <Container my={10}>
      <Text fontSize="2xl" fontWeight="bold" mb={4}>
        Trending Markets
      </Text>
      <HStack overflow="scroll">
        {data?.map((item) => <TrendingMarketCard market={item} />)}
      </HStack>
    </Container>
  );
}
