export interface BaseResponse {
  message: string;
  success: boolean;
}
export interface ErrorResponse {
  error: string;
}

export interface LoginResponse extends BaseResponse {
  userId: string;
  sessionToken: string;
}

export interface GetUserResponse {
  avatar: string;
  balance: number;
  email: string;
  name: string;
  public_key: string;
}
