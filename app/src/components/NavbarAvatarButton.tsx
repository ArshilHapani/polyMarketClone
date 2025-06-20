import {
  Avatar,
  Box,
  Flex,
  Icon,
  Popover,
  Portal,
  SkeletonCircle,
  Text,
} from "@chakra-ui/react";
import { LogOut, User2 } from "lucide-react";
import Link from "next/link";
import React, { useState } from "react";
import { googleLogout } from "@react-oauth/google";
import jsCookie from "js-cookie";

import useUserInfo from "@/hooks/useUserInfo";
import useRevalidation from "@/hooks/useRevalidate";
import GoogleSignInButton from "./GoogleSignInButton";

const NavbarAvatarButton = () => {
  const { data, isLoading } = useUserInfo();
  const [openPopover, setOpenPopover] = useState(false);
  const revalidate = useRevalidation();

  function handleLogout() {
    jsCookie.remove("polymarketAuthToken");
    googleLogout();
    queueMicrotask(() => revalidate(["userData"]));
    setOpenPopover(false);
  }
  if (isLoading) {
    return <SkeletonCircle size="9" />;
  }
  return (
    <div>
      {data && !isLoading ? (
        <Popover.Root
          size="sm"
          onOpenChange={(open) => setOpenPopover(open.open)}
          open={openPopover}
        >
          <Popover.Trigger>
            <Avatar.Root shape="full" size="sm">
              <Avatar.Fallback name={data.name} />
              <Avatar.Image src={data.avatar} />
            </Avatar.Root>
          </Popover.Trigger>

          <Portal>
            <Popover.Positioner>
              <Popover.Content>
                <Popover.Body padding={3}>
                  <Box spaceY={2}>
                    <Box
                      padding={2}
                      rounded="md"
                      _hover={{
                        backgroundColor: "gray.100",
                        cursor: "pointer",
                      }}
                    >
                      <Link href="/profile" className="flex items-center">
                        <Icon size="md">
                          <User2 />
                        </Icon>
                        <Text ml={2}>Profile</Text>
                      </Link>
                    </Box>
                    <Flex
                      padding={2}
                      rounded="md"
                      _hover={{
                        backgroundColor: "red.50",
                        cursor: "pointer",
                      }}
                      onClick={handleLogout}
                      as="button"
                      width="full"
                    >
                      <Icon size="md" color="red.500">
                        <LogOut />
                      </Icon>
                      <Text ml={2} color={"red.500"}>
                        Logout
                      </Text>
                    </Flex>
                  </Box>
                </Popover.Body>
              </Popover.Content>
            </Popover.Positioner>
          </Portal>
        </Popover.Root>
      ) : (
        <GoogleSignInButton />
      )}
    </div>
  );
};

export default NavbarAvatarButton;
