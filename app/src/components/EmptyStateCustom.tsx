import { EmptyState, VStack } from "@chakra-ui/react";
import { Scroll } from "lucide-react";
import React from "react";

type Props = {
  title?: string;
  description?: string;
};

const EmptyStateCustom = ({ description, title }: Props) => {
  return (
    <div>
      <EmptyState.Root>
        <EmptyState.Content>
          <EmptyState.Indicator>
            <Scroll />
          </EmptyState.Indicator>
          <VStack textAlign="center">
            <EmptyState.Title>{title}</EmptyState.Title>
            <EmptyState.Description>{description}</EmptyState.Description>
          </VStack>
        </EmptyState.Content>
      </EmptyState.Root>
    </div>
  );
};

export default EmptyStateCustom;
