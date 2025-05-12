import { GoogleLogin } from "@react-oauth/google";
import React from "react";

const GoogleSignInButton = () => {
  return (
    <GoogleLogin
      onSuccess={(credentialResponse) => {
        console.log(credentialResponse);
      }}
      onError={() => {
        console.log("Login Failed");
      }}
      logo_alignment="left"
    />
  );
};

export default GoogleSignInButton;
