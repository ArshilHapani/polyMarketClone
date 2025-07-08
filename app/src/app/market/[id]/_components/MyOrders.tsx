"use client";

import { useState } from "react";
import {
  LucideChevronLeft,
  LucideChevronRight,
  PenBoxIcon,
  X,
} from "lucide-react";
import { useMutation, useQuery } from "@tanstack/react-query";
import {
  Badge,
  Button,
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
import UpdateOrderModal from "@/components/modals/UpdateOrderModal";
import { toaster } from "@/components/ui/toaster";
import useModal from "@/hooks/useModal";
import useRevalidation from "@/hooks/useRevalidate";
import { OrderGetters } from "@/utils/interactions/dataGetter";
import { MarketActions } from "@/utils/interactions/dataPosters";
import { formatDate } from "@/utils";
import { OrderType } from "@/utils/types";

type Props = {
  marketId: string;
};

const MyOrders = ({ marketId }: Props) => {
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(["10"]);
  const { open } = useModal();
  const { mutateAsync } = useMutation({
    mutationFn: MarketActions.cancelOrder,
  });
  const revalidate = useRevalidation();
  const [filter, setFilter] = useState<OrderType>("open");

  const { data } = useQuery({
    queryKey: ["marketOrders", marketId, page, Number(pageSize), filter],
    queryFn: () =>
      OrderGetters.getUserOrdersByMarket(
        marketId,
        page,
        Number(pageSize[0] || 10),
        filter,
      ),
  });
  const clearFilterButton = (
    <Button
      variant="outline"
      rounded="full"
      onClick={() => {
        setFilter("open");
        setPage(1);
        setPageSize(["10"]);
      }}
      colorPalette="blue"
    >
      Clear filter
    </Button>
  );

  if (!data?.orders || data?.orders.length === 0) {
    return (
      <EmptyStateCustom
        title="No orders found"
        description="You have not placed any orders in this market yet."
        actionButton={clearFilterButton}
      />
    );
  }
  function handleCancelOrder(orderId: string) {
    const cnf = confirm(
      "Are you sure you want to cancel this order? This action cannot be undone.",
    );
    if (!cnf) return;
    toaster.promise(mutateAsync(orderId), {
      loading: { title: "Cancelling order..." },
      success: () => {
        revalidate(["marketOrders"]);
        return { title: "Order cancelled successfully!" };
      },
      error: (error) => ({
        title: "Error cancelling order",
        description: error instanceof Error ? error.message : "Unknown error",
        closable: true,
      }),
    });
  }
  return (
    <>
      <Stack width="full" gap="5">
        <Flex gap={3} alignItems="center" justifyContent="space-between">
          <Select.Root
            collection={orderFilters}
            size="sm"
            width="320px"
            onValueChange={(value) => setFilter(value.value[0] as OrderType)}
          >
            <Select.HiddenSelect />
            <Select.Control>
              <Select.Trigger>
                <Select.ValueText placeholder="Select order type" />
              </Select.Trigger>
              <Select.IndicatorGroup>
                <Select.Indicator />
              </Select.IndicatorGroup>
            </Select.Control>
            <Portal>
              <Select.Positioner>
                <Select.Content>
                  {orderFilters.items.map((filter) => (
                    <Select.Item item={filter} key={filter.value}>
                      {filter.label}
                      <Select.ItemIndicator />
                    </Select.Item>
                  ))}
                </Select.Content>
              </Select.Positioner>
            </Portal>
          </Select.Root>
          {filter !== "open" && clearFilterButton}
        </Flex>
        <Table.Root size="md" stickyHeader>
          <Table.Header>
            <Table.Row bg="bg.subtle">
              <Table.ColumnHeader></Table.ColumnHeader>
              <Table.ColumnHeader>Created at</Table.ColumnHeader>
              <Table.ColumnHeader>Price</Table.ColumnHeader>
              <Table.ColumnHeader>Quantity</Table.ColumnHeader>
              <Table.ColumnHeader>Outcome</Table.ColumnHeader>
              <Table.ColumnHeader>Side</Table.ColumnHeader>
              <Table.ColumnHeader>Status</Table.ColumnHeader>
              {filter === "open" && (
                <>
                  <Table.ColumnHeader>Update</Table.ColumnHeader>
                  <Table.ColumnHeader>Cancel</Table.ColumnHeader>
                </>
              )}
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
                      item.outcome === "yes" ? "green.600" : "red.600"
                    }
                    variant="solid"
                  >
                    {item.outcome}
                  </Badge>
                </Table.Cell>
                <Table.Cell>
                  <Badge
                    backgroundColor={
                      item.side === "buy" ? "green.600" : "red.600"
                    }
                    variant="solid"
                  >
                    {item.side}
                  </Badge>
                </Table.Cell>
                <Table.Cell>
                  <Badge
                    backgroundColor={
                      item.status === "OPEN" ? "blue.600" : "gray.600"
                    }
                    variant="solid"
                  >
                    {item.status}
                  </Badge>
                </Table.Cell>
                {filter === "open" && (
                  <>
                    <Table.Cell>
                      <IconButton
                        variant="ghost"
                        rounded="full"
                        colorPalette="blue"
                        color="blue.500"
                        onClick={() => open(`update-order-${item.id}`)}
                      >
                        <PenBoxIcon size={20} />
                      </IconButton>
                      <UpdateOrderModal
                        quantity={item.quantity}
                        filledQuantity={item.filled_quantity}
                        orderId={item.id}
                        price={item.price}
                      />
                    </Table.Cell>
                    <Table.Cell>
                      <IconButton
                        variant="ghost"
                        rounded="full"
                        colorPalette="red"
                        color="red.500"
                        onClick={() => handleCancelOrder(item.id)}
                      >
                        <X size={20} />
                      </IconButton>
                    </Table.Cell>
                  </>
                )}
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
                  disabled={data.orders.length < data.page_size}
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

const orderFilters = createListCollection({
  items: [
    {
      label: "Open",
      value: "open",
    },
    {
      label: "Cancelled",
      value: "cancelled",
    },
    {
      label: "Filled",
      value: "filled",
    },
    {
      label: "Expired",
      value: "expired",
    },
    {
      label: "Pending Update",
      value: "pending_update",
    },
    {
      label: "Pending Cancel",
      value: "pending_cancel",
    },
  ] as Readonly<{
    label: string;
    value: OrderType;
  }>[],
});
