"use client";

import React, { useEffect, useRef, useState } from "react";
import {
  Box,
  Button,
  Flex,
  Tabs,
  useDisclosure,
  CloseButton,
} from "@chakra-ui/react";
import TradeForm from "./TradeForm";

const PurchaseNowActionBar = () => {
  const { open: isOpen, onToggle } = useDisclosure();
  const ctnRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (ctnRef.current && !ctnRef.current.contains(event.target as Node)) {
        onToggle();
      }
    };

    document.addEventListener("click", handleClickOutside);
    return () => {
      document.removeEventListener("click", handleClickOutside);
    };
  }, []);

  return (
    <Box
      position="fixed"
      left={0}
      right={0}
      bottom={5}
      zIndex={1400}
      width={isOpen ? "400px" : "140px"}
      minHeight="80px"
      mx="auto"
      overflow="hidden"
      transition="all 0.2s ease-in-out"
    >
      {!isOpen ? (
        <Button
          onClick={onToggle}
          width="100%"
          size="lg"
          bg="blue.subtle/50"
          backdropBlur="md"
          backdropFilter="blur(10px)"
          variant="outline"
          rounded="full"
        >
          Trade Now
        </Button>
      ) : (
        <Box
          bg="gray.subtle/50"
          backdropBlur="md"
          backdropFilter="blur(10px)"
          boxShadow="0 -2px 8px rgba(0,0,0,0.08)"
          px={6}
          py={4}
          borderRadius="xl"
          minHeight="250px"
          _hover={{ boxShadow: "0 -4px 12px rgba(0,0,0,0.1)" }}
          ref={ctnRef}
        >
          <Tabs.Root defaultValue="buy">
            <Tabs.List
              justifyContent={"space-between"}
              display="flex"
              alignItems="center"
              gap={2}
            >
              <Flex>
                <Tabs.Trigger value="buy">Buy</Tabs.Trigger>
                <Tabs.Trigger value="sell">Sell</Tabs.Trigger>
              </Flex>
              <CloseButton onClick={onToggle} />
            </Tabs.List>
            <Tabs.Content value="buy">
              <TradeForm mode="buy" />
            </Tabs.Content>
            <Tabs.Content value="sell">
              <TradeForm mode="sell" />
            </Tabs.Content>
          </Tabs.Root>
        </Box>
      )}
    </Box>
  );
};

export default PurchaseNowActionBar;
