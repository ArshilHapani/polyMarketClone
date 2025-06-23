"use client";

import { useState } from "react";
import { Trash2 } from "lucide-react";
import { useQuery } from "@tanstack/react-query";
import { Badge, IconButton, Table } from "@chakra-ui/react";

import EmptyStateCustom from "@/components/EmptyStateCustom";
import { OrderGetters } from "@/utils/interactions/dataGetter";
import { formatDate } from "@/utils";

type Props = {
  marketId: string;
};

const MyOrders = ({ marketId }: Props) => {
  const [page, setPage] = useState(1);
  const { data } = useQuery({
    queryKey: ["marketOrders", marketId, page],
    queryFn: () => OrderGetters.getUserOrdersByMarket(marketId, page, 10),
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
    <Table.ScrollArea borderWidth="1px" rounded="md" maxHeight="500px">
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
                    item.side === "BUY" ? "green.600" : "red.600"
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
    </Table.ScrollArea>
  );
};

export default MyOrders;
