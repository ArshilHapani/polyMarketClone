"use client";

import GoogleSignInButton from "@/components/GoogleSignInButton";
import { Container } from "@chakra-ui/react";

export default function Home() {
  return (
    <Container my={10}>
      <GoogleSignInButton />
    </Container>
  );
}
