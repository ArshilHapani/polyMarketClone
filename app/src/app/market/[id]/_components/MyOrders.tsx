"use client";

import { useState } from "react";
import { LucideChevronLeft, LucideChevronRight, Trash2 } from "lucide-react";
import { useQuery } from "@tanstack/react-query";
import {
  Badge,
  ButtonGroup,
  createListCollection,
  Flex,
  IconButton,
  Pagination,
  Portal,
  Select,
  Stack,
  Table,
} from "@chakra-ui/react";

import EmptyStateCustom from "@/components/EmptyStateCustom";
import { OrderGetters } from "@/utils/interactions/dataGetter";
import { formatDate } from "@/utils";

type Props = {
  marketId: string;
};

const MyOrders = ({ marketId }: Props) => {
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(["10"]);
  const { data } = useQuery({
    queryKey: ["marketOrders", marketId, page, Number(pageSize)],
    queryFn: () =>
      OrderGetters.getUserOrdersByMarket(
        marketId,
        page,
        Number(pageSize[0] || 10),
      ),
  });

  if (!data?.orders || data?.orders.length === 0) {
    return (
      <EmptyStateCustom
        title="No orders found"
        description="You have not placed any orders in this market yet."
      />
    );
  }
  return (
    <>
      <Stack width="full" gap="5">
        <Table.Root size="md" stickyHeader>
          <Table.Header>
            <Table.Row bg="bg.subtle">
              <Table.ColumnHeader></Table.ColumnHeader>
              <Table.ColumnHeader>Created at</Table.ColumnHeader>
              <Table.ColumnHeader>Price</Table.ColumnHeader>
              <Table.ColumnHeader>Quantity</Table.ColumnHeader>
              <Table.ColumnHeader>Outcome</Table.ColumnHeader>
              <Table.ColumnHeader>Side</Table.ColumnHeader>
              <Table.ColumnHeader>Delete</Table.ColumnHeader>
            </Table.Row>
          </Table.Header>

          <Table.Body>
            {data.orders.map((item, idx) => (
              <Table.Row key={item.id}>
                <Table.Cell>{idx + 1}</Table.Cell>
                <Table.Cell>{formatDate(item.created_at)}</Table.Cell>
                <Table.Cell>{item.price}</Table.Cell>
                <Table.Cell>{item.quantity}</Table.Cell>

                <Table.Cell>
                  <Badge
                    backgroundColor={
                      item.outcome === "YES" ? "green.600" : "red.600"
                    }
                    variant="solid"
                  >
                    {item.outcome}
                  </Badge>
                </Table.Cell>
                <Table.Cell>
                  <Badge
                    backgroundColor={
                      item.side === "BUY" ? "green.600" : "red.600"
                    }
                    variant="solid"
                  >
                    {item.side}
                  </Badge>
                </Table.Cell>
                <Table.Cell>
                  <IconButton
                    variant="ghost"
                    rounded="full"
                    colorPalette="red"
                    color="red.500"
                    onClick={() => {
                      console.log(`Delete order with id: ${item.id}`);
                    }}
                  >
                    <Trash2 size={20} />
                  </IconButton>
                </Table.Cell>
              </Table.Row>
            ))}
          </Table.Body>
        </Table.Root>
        <Flex
          justifyContent="flex-end"
          width="full"
          alignItems="center"
          gap={3}
        >
          <Select.Root
            collection={sizes}
            size="sm"
            width="120px"
            value={pageSize}
            onValueChange={(value) => setPageSize(value.value)}
          >
            <Select.HiddenSelect />
            <Select.Control>
              <Select.Trigger>
                <Select.ValueText placeholder="Page size" />
              </Select.Trigger>
              <Select.IndicatorGroup>
                <Select.Indicator />
              </Select.IndicatorGroup>
            </Select.Control>
            <Portal>
              <Select.Positioner>
                <Select.Content>
                  {sizes.items.map((size) => (
                    <Select.Item item={size} key={size.value}>
                      {size.label}
                      <Select.ItemIndicator />
                    </Select.Item>
                  ))}
                </Select.Content>
              </Select.Positioner>
            </Portal>
          </Select.Root>
          <Pagination.Root pageSize={data.page_size} page={data.page}>
            <ButtonGroup variant="ghost" size="sm" wrap="wrap">
              <Pagination.PrevTrigger asChild>
                <IconButton
                  onClick={() => setPage((prev) => Math.max(prev - 1, 1))}
                  disabled={data.page === 1}
                >
                  <LucideChevronLeft />
                </IconButton>
              </Pagination.PrevTrigger>

              <IconButton variant={{ base: "ghost", _selected: "outline" }}>
                {data.page}
              </IconButton>

              <Pagination.NextTrigger asChild>
                <IconButton
                  onClick={() => setPage((prev) => prev + 1)}
                  disabled={data.page === data.page_size}
                >
                  <LucideChevronRight />
                </IconButton>
              </Pagination.NextTrigger>
            </ButtonGroup>
          </Pagination.Root>
        </Flex>
      </Stack>
    </>
  );
};

export default MyOrders;
const sizes = createListCollection({
  items: [
    { label: 10, value: 10 },
    { label: 20, value: 20 },
    { label: 50, value: 50 },
    { label: 100, value: 100 },
  ],
});
