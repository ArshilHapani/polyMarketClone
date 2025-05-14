"use client";

import { Container } from "@chakra-ui/react";
import { useQuery } from "@tanstack/react-query";

import GoogleSignInButton from "@/components/GoogleSignInButton";
import { MarketGetters } from "@/utils/interactions/dataGetter";

export default function Home() {
  const { data, error } = useQuery({
    queryFn: () => MarketGetters.getMarketData(1, 10),
    queryKey: ["marketData", 1, 10],
  });
  return (
    <Container my={10}>
      <pre>{JSON.stringify(data, null, 2)}</pre>
      <h1>Error</h1>
      <pre>{JSON.stringify(error, null, 2)}</pre>
      <GoogleSignInButton />
    </Container>
  );
}
