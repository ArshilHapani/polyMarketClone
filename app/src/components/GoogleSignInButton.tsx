import { UserAuthActions } from "@/utils/interactions/dataPosters";
import { GoogleLogin, googleLogout } from "@react-oauth/google";
import { useMutation } from "@tanstack/react-query";
import React from "react";
import cookie from "js-cookie";

import { toaster } from "./ui/toaster";

const GoogleSignInButton = () => {
  const { mutateAsync } = useMutation({
    mutationFn: UserAuthActions.handleSignInWithGoogle,
  });

  function handleLogin(loginId: string) {
    console.log({ loginId });
    toaster.promise(mutateAsync({ id_token: loginId }), {
      error(arg: any) {
        return {
          title: "Error",
          description: arg?.message || "Failed to login with google",
        };
      },
      success(arg) {
        cookie.set("polymarketAuthToken", arg.sessionToken, {
          expires: 60 * 60 * 24 * 30, // 30 days,
          secure: true,
        });
        return {
          title: "Success",
          description: "Welcome to polymarket",
        };
      },
      loading: {
        title: "Waiting for sign in...",
        description: "Please complete your sign in process in popup window",
      },
    });
  }
  return (
    <>
      <GoogleLogin
        onSuccess={(credentialResponse) => {
          if (!credentialResponse.credential) {
            toaster.error({ title: "Failed to get credentials from google" });
            return;
          }
          handleLogin(credentialResponse.credential);
        }}
        onError={() => {
          console.log("Login Failed");
        }}
        logo_alignment="center"
        shape="pill"
        size="large"
      />
    </>
  );
};

export default GoogleSignInButton;
