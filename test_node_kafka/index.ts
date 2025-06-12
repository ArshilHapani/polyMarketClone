import { Kafka, Partitioners, type ProducerRecord } from "kafkajs";

const kafka = new Kafka({
  brokers: ["localhost:9092"],
  clientId: "test-node-kafka",
});

const producer = kafka.producer({
  allowAutoTopicCreation: true,
  createPartitioner: Partitioners.LegacyPartitioner,
});

await producer.connect();

console.log("Producer connected");

const topic = "test-node-kafka-topic";

for (let i = 0; i < 100; i++) {
  const message = "A new price update is available!" + i;
  const record: ProducerRecord = {
    messages: [{ value: message }],
    topic,
  };
  const rx = await producer.send(record);
  console.log("Message sent:", rx);
}

await producer.disconnect();
process.exit();
