![Architecture v1](./assets/architecture_1.png)

# Websocket subscription management architecture

Each channel -> separate process

Task -
channel wise tasks
parameters: - data sended by subscribing channel (market id or other parameters) - event transmitter (tx) via which processed data will be sent

subscription mechanism:
Hashmap<Channel, Vec<UserTransmitter>>

when we receive message from tasks's transmitter for particular channel, then we broadcast message to all UserTransmitter in particular subscriber's channel
