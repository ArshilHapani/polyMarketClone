import { Avatar, Button, Card } from "@chakra-ui/react";
import Link from "next/link";
import React from "react";

import { Market } from "@/generated/grpc_service_types/markets";

type Props = {
  market: Market;
};

const TrendingMarketCard = ({ market }: Props) => {
  return (
    <Link href={`/market/${market.id}`}>
      <Card.Root width="320px" height="220px">
        <Card.Body gap="2">
          <Avatar.Root size="lg" shape="rounded">
            <Avatar.Image src={market.logo} />
            <Avatar.Fallback name={market.name} />
          </Avatar.Root>
          <Card.Title truncate mt="2">
            {market.name}
          </Card.Title>
        </Card.Body>
        <Card.Footer justifyContent="flex-end">
          <Button variant="outline" onClick={(e) => e.stopPropagation()}>
            Buy Yes
          </Button>
          <Button onClick={(e) => e.stopPropagation()}>Buy No</Button>
        </Card.Footer>
      </Card.Root>
    </Link>
  );
};

export default TrendingMarketCard;
