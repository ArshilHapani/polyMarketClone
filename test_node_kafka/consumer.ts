import { Kafka } from "kafkajs";

const kafka = new Kafka({
  brokers: ["localhost:9092"],
  clientId: "test-node-kafka",
});

const consumer = kafka.consumer({
  groupId: "test-node-kafka-group",
});

await consumer.connect();
console.log("Consumer connected");

const topic = "test-node-kafka-topic";

await consumer.subscribe({ topic, fromBeginning: true });
console.log(`Subscribed to topic: ${topic}`);

await consumer.run({
  eachMessage: async ({ topic, partition, message }) => {
    console.log(
      `Received message: ${message?.value?.toString()} from topic: ${topic}, partition: ${partition}`
    );
  },
});
