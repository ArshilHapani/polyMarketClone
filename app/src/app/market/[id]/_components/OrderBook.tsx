import { Box, Table, Badge, Text } from "@chakra-ui/react";

type Props = {
  tradeType: "yes" | "no";
};

const OrderBook = ({ tradeType }: Props) => {
  const buyOrders = generateDummyOrderBookData(10, true);
  const sellOrders = generateDummyOrderBookData(10);

  const sortedBuyOrders = [...buyOrders].sort((a, b) => b.price - a.price);
  const sortedSellOrders = [...sellOrders].sort((a, b) => a.price - b.price);
  const firstBuyAndAfterThatSellOrdersInSortedAsPerPrice = [
    ...sortedBuyOrders,
    ...sortedSellOrders,
  ];

  const buyTotalUsers = buyOrders.reduce((acc, order) => acc + order.users, 0);
  const sellTotalUsers = sellOrders.reduce(
    (acc, order) => acc + order.users,
    0,
  );

  return (
    <Box>
      <Table.ScrollArea borderWidth="1px" rounded="md">
        <Table.Root size="sm" stickyHeader bg="transparent" variant="outline">
          <Table.Header>
            <Table.Row bg="transparent">
              <Table.ColumnHeader>Trade {tradeType}</Table.ColumnHeader>
              <Table.ColumnHeader>Price</Table.ColumnHeader>
              <Table.ColumnHeader>Quantity</Table.ColumnHeader>
              <Table.ColumnHeader>Total</Table.ColumnHeader>
            </Table.Row>
          </Table.Header>

          <Table.Body>
            {firstBuyAndAfterThatSellOrdersInSortedAsPerPrice.map(
              (order, idx) => {
                let midContent: React.ReactNode = null;
                if (idx === sortedBuyOrders.length) {
                  const bestBuy = sortedBuyOrders[0]?.price ?? 0;
                  const bestSell = sortedSellOrders[0]?.price ?? 0;
                  const spread = (bestSell - bestBuy).toFixed(2);

                  midContent = (
                    <Table.Row key="spread-row" border="none">
                      <Table.Cell colSpan={4} textAlign="center" py={2}>
                        <Text fontWeight="bold" color="gray.600" fontSize="sm">
                          Spread: {spread}
                        </Text>
                      </Table.Cell>
                    </Table.Row>
                  );
                }

                return (
                  <>
                    {midContent}
                    <Table.Row
                      key={order.price + idx}
                      border="none"
                      _hover={{
                        bg: order.type === "buy" ? "green.100/60" : "red.50/60",
                      }}
                    >
                      <Table.Cell padding={0} position="relative" height="100%">
                        <Box
                          position="absolute"
                          left={0}
                          top={0}
                          bottom={0}
                          width={getBarPercentage(
                            order.users,
                            order.type === "buy"
                              ? buyTotalUsers
                              : sellTotalUsers,
                          )}
                          bg={
                            order.type == "buy" ? "green.500/30" : "red.500/30"
                          }
                          height="100%"
                          zIndex={0}
                        />
                        <Box
                          position="relative"
                          zIndex={1}
                          display="flex"
                          alignItems="center"
                          height="25px"
                        >
                          {(idx === sortedBuyOrders.length - 1 ||
                            idx === sortedBuyOrders.length) && (
                            <Badge
                              bg={
                                order.type === "buy" ? "green.500" : "red.500"
                              }
                              color="white"
                              ml={4}
                            >
                              {order.type === "buy" ? "Bids" : "Asks"}
                            </Badge>
                          )}
                        </Box>
                      </Table.Cell>
                      <Table.Cell>{order.price}</Table.Cell>
                      <Table.Cell>{order.shares}</Table.Cell>
                      <Table.Cell>{order.total}</Table.Cell>
                    </Table.Row>
                  </>
                );
              },
            )}
          </Table.Body>
        </Table.Root>
      </Table.ScrollArea>
    </Box>
  );
};

export default OrderBook;

type OrderBookLevel = {
  price: number;
  shares: number;
  total: number;
  users: number;
  type: "buy" | "sell";
};
function generateDummyOrderBookData(
  levels: number = 5,
  isBuy: boolean = false,
): OrderBookLevel[] {
  let data: OrderBookLevel[] = [];
  let total = 0;

  // Generate prices in sorted order (descending for buy, ascending for sell)
  let prices = Array.from(
    { length: levels },
    () => +(Math.random() * 0.5 + 0.5).toFixed(2),
  );
  prices.sort((a, b) => (isBuy ? b - a : a - b));

  for (let i = 0; i < levels; i++) {
    const price = prices[i];
    const shares = Math.floor(Math.random() * 100 + 1); // shares between 1 and 100
    total += shares;

    // Users: descending for buy (high price = more users), ascending for sell (low price = more users)
    const maxUsers = 10;
    const minUsers = 1;
    let users: number;
    if (isBuy) {
      users = Math.round(maxUsers - ((maxUsers - minUsers) * i) / (levels - 1));
    } else {
      users = Math.round(minUsers + ((maxUsers - minUsers) * i) / (levels - 1));
    }

    data.push({ price, shares, total, users, type: isBuy ? "buy" : "sell" });
  }
  return data;
}

function getBarPercentage(users: number, totalUsers: number): string {
  if (totalUsers === 0) return "0%";
  return ((users / totalUsers) * 100).toFixed(2) + "%";
}
