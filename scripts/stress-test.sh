#!/bin/bash

# Tokens
TOKEN_ONE="eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJ1c2VyX2lkIjoiNTkzYzA4ZjAtNjY5NS00YjQyLTg2ZjEtNTQ2ZTU1NTMwMTFjIiwiZ29vZ2xlX3N1YiI6IjEwNjM4NzY5OTc0NDM1NTA5NTc1NiIsImVtYWlsIjoiYXJzaGlsaGFwYW5pOTk4QGdtYWlsLmNvbSIsImV4cCI6MTc1MTcwNTY4NX0.Z_7u1tKQ2GhvXR2IPxgE-yYTloJ7BkrP1gjZNJCRSx4"
TOKEN_TWO="eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJ1c2VyX2lkIjoiNGYyZjU4MzItZjgzZS00YzI1LWFhMTQtZDYyZWZjNTJlOTEyIiwiZ29vZ2xlX3N1YiI6IjEwMTgwMzQwNDA0NTQ3OTg1MjIxMCIsImVtYWlsIjoiYXJzaGlsaGFwYW5pMjAwNEBnbWFpbC5jb20iLCJleHAiOjE3NTE3MTE3MTZ9.LlvAMTHxg0AXLMlyQXGnKkP_kMDRo6It_KeCC-MrFXw"

# Config
URL="http://localhost:8080/user/orders/create"
REQ_COUNT=500
CONCURRENCY=50

# Headers
HEADER_ONE="Authorization: Bearer $TOKEN_ONE"
HEADER_TWO="Authorization: Bearer $TOKEN_TWO"
CONTENT_TYPE="Content-Type: application/json"

# Payloads
cat > payload1.json <<EOF
{
  "market_id": "898a074c-48da-49e7-90f4-417e6e5e5886",
  "price": 0.4,
  "quantity": 12,
  "side": "BUY",
  "outcome_side": "YES"
}
EOF

cat > payload2.json <<EOF
{
  "market_id": "898a074c-48da-49e7-90f4-417e6e5e5886",
  "price": 0.34,
  "quantity": 12,
  "side": "SELL",
  "outcome_side": "YES"
}
EOF

# Run hey for both payloads concurrently
echo "Starting concurrent stress test with 2 payloads..."

hey -n $REQ_COUNT -c $CONCURRENCY -m POST \
  -H "$HEADER_ONE" \
  -H "$CONTENT_TYPE" \
  -d @payload1.json \
  "$URL" > result_buy.txt &

hey -n $REQ_COUNT -c $CONCURRENCY -m POST \
  -H "$HEADER_TWO" \
  -H "$CONTENT_TYPE" \
  -d @payload2.json \
  "$URL" > result_sell.txt &

# Wait for both to finish
wait

echo "Stress test complete. Results saved to result_buy.txt and result_sell.txt"