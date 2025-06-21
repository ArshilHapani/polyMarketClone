import React from "react";
import { Avatar, Box, Container, Flex, Text } from "@chakra-ui/react";

import EmptyStateCustom from "@/components/EmptyStateCustom";
import { MarketGetters } from "@/utils/interactions/dataGetter";
import { Bookmark, Clock5, Link } from "lucide-react";
import PriceChart from "./_components/PriceChart";
import PurchaseNowActionBar from "./_components/PurchaseNowActionBar";

type Props = {
  params: Promise<{
    id: string;
  }>;
};

const MarketPage = async ({ params }: Props) => {
  const id = (await params).id;
  const market = await MarketGetters.getMarketById(id);

  if (!market) {
    return (
      <EmptyStateCustom
        title="Market not found"
        description="Please cross check the url as the market which you are looking is not found"
      />
    );
  }
  return (
    <Container my={10}>
      <Box mb={20}>
        {/* avatar and title flex */}
        <Flex alignItems="center" gap={3}>
          <Avatar.Root size="2xl" shape="rounded">
            <Avatar.Image src={market.logo} />
            <Avatar.Fallback name={market.name} />
          </Avatar.Root>
          <Text fontSize="2xl" fontWeight="semibold">
            {market.name}
          </Text>
        </Flex>
        {/* volume and links */}
        <Flex mt={4} justifyContent="space-between">
          <Flex alignItems="center" gap={2}>
            <Text color="gray.600" fontSize="sm">
              $84,424 Vol.
            </Text>
            <Flex color="gray.600" fontSize="sm" alignItems={"center"} gap={1}>
              <Clock5 size={14} />
              <Text>
                {new Date(market.marketExpiry).toLocaleDateString("en-US", {
                  year: "numeric",
                  month: "long",
                  day: "numeric",
                })}
              </Text>
            </Flex>
          </Flex>
          <Flex alignItems="center" gap={3} color="gray.800">
            <Link size={16} />
            <Bookmark size={16} />
          </Flex>
        </Flex>

        {/* charts */}
        <PriceChart />

        {/*  action bar for purchasing now  */}
        <PurchaseNowActionBar />
      </Box>
    </Container>
  );
};

export default MarketPage;
