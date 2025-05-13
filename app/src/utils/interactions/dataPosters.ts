import axios from "axios";
import { LoginResponse } from "../types/api";

const BASE_URL = process.env.NEXT_PUBLIC_SERVICE_API_URL;

export class UserAuthActions {
  static async handleSignInWithGoogle({ id_token }: { id_token: string }) {
    const { data, status } = await axios.post(`${BASE_URL}/login`, {
      id_token,
    });

    if (status != 200) throw new Error(data.error);
    return data as LoginResponse;
  }
}
