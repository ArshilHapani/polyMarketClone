import React from "react";
import { Avatar, Box, Container, Flex, Tabs, Text } from "@chakra-ui/react";
import { Bookmark, Clock5, Link } from "lucide-react";

import EmptyStateCustom from "@/components/EmptyStateCustom";
import { MarketGetters } from "@/utils/interactions/dataGetter";
import PriceChart from "./_components/PriceChart";
import PurchaseNowActionBar from "./_components/PurchaseNowActionBar";
import OrderBook from "./_components/OrderBook";
import MyOrders from "./_components/MyOrders";

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
        <PriceChart market_id={id} />

        {/*  action bar for purchasing now  */}
        <PurchaseNowActionBar market_id={id} />

        {/* order book */}
        <Box mt={10}>
          <Tabs.Root defaultValue="yes_book">
            <Tabs.List>
              <Tabs.Trigger value="yes_book">Trade yes</Tabs.Trigger>
              <Tabs.Trigger value="no_book">Trade no</Tabs.Trigger>
              <Tabs.Trigger value="my_orders">My orders</Tabs.Trigger>
              <Tabs.Trigger value="top_holders">Top holders</Tabs.Trigger>
            </Tabs.List>
            <Tabs.Content value="yes_book">
              <OrderBook tradeType="yes" marketId={id} />
            </Tabs.Content>
            <Tabs.Content value="no_book">
              <OrderBook tradeType="no" marketId={id} />
            </Tabs.Content>
            <Tabs.Content value="my_orders">
              <MyOrders marketId={id} />
            </Tabs.Content>
            <Tabs.Content value="top_holders">
              <Text fontSize="lg" fontWeight="bold" mb={4}>
                Top Holders
              </Text>
              <Text color="gray.600" fontSize="sm">
                Coming soon...
              </Text>
            </Tabs.Content>
          </Tabs.Root>
        </Box>
      </Box>
    </Container>
  );
};

export default MarketPage;
