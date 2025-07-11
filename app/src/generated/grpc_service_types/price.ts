// @generated by protobuf-ts 2.10.0 with parameter generate_dependencies,long_type_number
// @generated from protobuf file "price.proto" (package "price", syntax proto3)
// tslint:disable
import { ServiceType } from "@protobuf-ts/runtime-rpc";
import type { BinaryWriteOptions } from "@protobuf-ts/runtime";
import type { IBinaryWriter } from "@protobuf-ts/runtime";
import { WireType } from "@protobuf-ts/runtime";
import type { BinaryReadOptions } from "@protobuf-ts/runtime";
import type { IBinaryReader } from "@protobuf-ts/runtime";
import { UnknownFieldHandler } from "@protobuf-ts/runtime";
import type { PartialMessage } from "@protobuf-ts/runtime";
import { reflectionMergePartial } from "@protobuf-ts/runtime";
import { MessageType } from "@protobuf-ts/runtime";
import { Timeframe } from "./common";
/**
 * @generated from protobuf message price.GetPriceDataWithinIntervalRequest
 */
export interface GetPriceDataWithinIntervalRequest {
    /**
     * @generated from protobuf field: string market_id = 1;
     */
    marketId: string;
    /**
     * @generated from protobuf field: common.Timeframe timeframe = 2;
     */
    timeframe: Timeframe;
}
/**
 * @generated from protobuf message price.PriceData
 */
export interface PriceData {
    /**
     * @generated from protobuf field: double yes_price = 1;
     */
    yesPrice: number;
    /**
     * @generated from protobuf field: double no_price = 2;
     */
    noPrice: number;
    /**
     * @generated from protobuf field: uint64 timestamp = 3;
     */
    timestamp: number;
}
/**
 * @generated from protobuf message price.GetMarketPriceDataWithinIntervalResponse
 */
export interface GetMarketPriceDataWithinIntervalResponse {
    /**
     * @generated from protobuf field: repeated price.PriceData price_data = 1;
     */
    priceData: PriceData[];
    /**
     * @generated from protobuf field: string market_id = 2;
     */
    marketId: string;
}
// @generated message type with reflection information, may provide speed optimized methods
class GetPriceDataWithinIntervalRequest$Type extends MessageType<GetPriceDataWithinIntervalRequest> {
    constructor() {
        super("price.GetPriceDataWithinIntervalRequest", [
            { no: 1, name: "market_id", kind: "scalar", T: 9 /*ScalarType.STRING*/ },
            { no: 2, name: "timeframe", kind: "enum", T: () => ["common.Timeframe", Timeframe, "TIMEFRAME_"] }
        ]);
    }
    create(value?: PartialMessage<GetPriceDataWithinIntervalRequest>): GetPriceDataWithinIntervalRequest {
        const message = globalThis.Object.create((this.messagePrototype!));
        message.marketId = "";
        message.timeframe = 0;
        if (value !== undefined)
            reflectionMergePartial<GetPriceDataWithinIntervalRequest>(this, message, value);
        return message;
    }
    internalBinaryRead(reader: IBinaryReader, length: number, options: BinaryReadOptions, target?: GetPriceDataWithinIntervalRequest): GetPriceDataWithinIntervalRequest {
        let message = target ?? this.create(), end = reader.pos + length;
        while (reader.pos < end) {
            let [fieldNo, wireType] = reader.tag();
            switch (fieldNo) {
                case /* string market_id */ 1:
                    message.marketId = reader.string();
                    break;
                case /* common.Timeframe timeframe */ 2:
                    message.timeframe = reader.int32();
                    break;
                default:
                    let u = options.readUnknownField;
                    if (u === "throw")
                        throw new globalThis.Error(`Unknown field ${fieldNo} (wire type ${wireType}) for ${this.typeName}`);
                    let d = reader.skip(wireType);
                    if (u !== false)
                        (u === true ? UnknownFieldHandler.onRead : u)(this.typeName, message, fieldNo, wireType, d);
            }
        }
        return message;
    }
    internalBinaryWrite(message: GetPriceDataWithinIntervalRequest, writer: IBinaryWriter, options: BinaryWriteOptions): IBinaryWriter {
        /* string market_id = 1; */
        if (message.marketId !== "")
            writer.tag(1, WireType.LengthDelimited).string(message.marketId);
        /* common.Timeframe timeframe = 2; */
        if (message.timeframe !== 0)
            writer.tag(2, WireType.Varint).int32(message.timeframe);
        let u = options.writeUnknownFields;
        if (u !== false)
            (u == true ? UnknownFieldHandler.onWrite : u)(this.typeName, message, writer);
        return writer;
    }
}
/**
 * @generated MessageType for protobuf message price.GetPriceDataWithinIntervalRequest
 */
export const GetPriceDataWithinIntervalRequest = new GetPriceDataWithinIntervalRequest$Type();
// @generated message type with reflection information, may provide speed optimized methods
class PriceData$Type extends MessageType<PriceData> {
    constructor() {
        super("price.PriceData", [
            { no: 1, name: "yes_price", kind: "scalar", T: 1 /*ScalarType.DOUBLE*/ },
            { no: 2, name: "no_price", kind: "scalar", T: 1 /*ScalarType.DOUBLE*/ },
            { no: 3, name: "timestamp", kind: "scalar", T: 4 /*ScalarType.UINT64*/, L: 2 /*LongType.NUMBER*/ }
        ]);
    }
    create(value?: PartialMessage<PriceData>): PriceData {
        const message = globalThis.Object.create((this.messagePrototype!));
        message.yesPrice = 0;
        message.noPrice = 0;
        message.timestamp = 0;
        if (value !== undefined)
            reflectionMergePartial<PriceData>(this, message, value);
        return message;
    }
    internalBinaryRead(reader: IBinaryReader, length: number, options: BinaryReadOptions, target?: PriceData): PriceData {
        let message = target ?? this.create(), end = reader.pos + length;
        while (reader.pos < end) {
            let [fieldNo, wireType] = reader.tag();
            switch (fieldNo) {
                case /* double yes_price */ 1:
                    message.yesPrice = reader.double();
                    break;
                case /* double no_price */ 2:
                    message.noPrice = reader.double();
                    break;
                case /* uint64 timestamp */ 3:
                    message.timestamp = reader.uint64().toNumber();
                    break;
                default:
                    let u = options.readUnknownField;
                    if (u === "throw")
                        throw new globalThis.Error(`Unknown field ${fieldNo} (wire type ${wireType}) for ${this.typeName}`);
                    let d = reader.skip(wireType);
                    if (u !== false)
                        (u === true ? UnknownFieldHandler.onRead : u)(this.typeName, message, fieldNo, wireType, d);
            }
        }
        return message;
    }
    internalBinaryWrite(message: PriceData, writer: IBinaryWriter, options: BinaryWriteOptions): IBinaryWriter {
        /* double yes_price = 1; */
        if (message.yesPrice !== 0)
            writer.tag(1, WireType.Bit64).double(message.yesPrice);
        /* double no_price = 2; */
        if (message.noPrice !== 0)
            writer.tag(2, WireType.Bit64).double(message.noPrice);
        /* uint64 timestamp = 3; */
        if (message.timestamp !== 0)
            writer.tag(3, WireType.Varint).uint64(message.timestamp);
        let u = options.writeUnknownFields;
        if (u !== false)
            (u == true ? UnknownFieldHandler.onWrite : u)(this.typeName, message, writer);
        return writer;
    }
}
/**
 * @generated MessageType for protobuf message price.PriceData
 */
export const PriceData = new PriceData$Type();
// @generated message type with reflection information, may provide speed optimized methods
class GetMarketPriceDataWithinIntervalResponse$Type extends MessageType<GetMarketPriceDataWithinIntervalResponse> {
    constructor() {
        super("price.GetMarketPriceDataWithinIntervalResponse", [
            { no: 1, name: "price_data", kind: "message", repeat: 2 /*RepeatType.UNPACKED*/, T: () => PriceData },
            { no: 2, name: "market_id", kind: "scalar", T: 9 /*ScalarType.STRING*/ }
        ]);
    }
    create(value?: PartialMessage<GetMarketPriceDataWithinIntervalResponse>): GetMarketPriceDataWithinIntervalResponse {
        const message = globalThis.Object.create((this.messagePrototype!));
        message.priceData = [];
        message.marketId = "";
        if (value !== undefined)
            reflectionMergePartial<GetMarketPriceDataWithinIntervalResponse>(this, message, value);
        return message;
    }
    internalBinaryRead(reader: IBinaryReader, length: number, options: BinaryReadOptions, target?: GetMarketPriceDataWithinIntervalResponse): GetMarketPriceDataWithinIntervalResponse {
        let message = target ?? this.create(), end = reader.pos + length;
        while (reader.pos < end) {
            let [fieldNo, wireType] = reader.tag();
            switch (fieldNo) {
                case /* repeated price.PriceData price_data */ 1:
                    message.priceData.push(PriceData.internalBinaryRead(reader, reader.uint32(), options));
                    break;
                case /* string market_id */ 2:
                    message.marketId = reader.string();
                    break;
                default:
                    let u = options.readUnknownField;
                    if (u === "throw")
                        throw new globalThis.Error(`Unknown field ${fieldNo} (wire type ${wireType}) for ${this.typeName}`);
                    let d = reader.skip(wireType);
                    if (u !== false)
                        (u === true ? UnknownFieldHandler.onRead : u)(this.typeName, message, fieldNo, wireType, d);
            }
        }
        return message;
    }
    internalBinaryWrite(message: GetMarketPriceDataWithinIntervalResponse, writer: IBinaryWriter, options: BinaryWriteOptions): IBinaryWriter {
        /* repeated price.PriceData price_data = 1; */
        for (let i = 0; i < message.priceData.length; i++)
            PriceData.internalBinaryWrite(message.priceData[i], writer.tag(1, WireType.LengthDelimited).fork(), options).join();
        /* string market_id = 2; */
        if (message.marketId !== "")
            writer.tag(2, WireType.LengthDelimited).string(message.marketId);
        let u = options.writeUnknownFields;
        if (u !== false)
            (u == true ? UnknownFieldHandler.onWrite : u)(this.typeName, message, writer);
        return writer;
    }
}
/**
 * @generated MessageType for protobuf message price.GetMarketPriceDataWithinIntervalResponse
 */
export const GetMarketPriceDataWithinIntervalResponse = new GetMarketPriceDataWithinIntervalResponse$Type();
/**
 * @generated ServiceType for protobuf service price.PriceService
 */
export const PriceService = new ServiceType("price.PriceService", [
    { name: "GetPriceDataWithinInterval", options: {}, I: GetPriceDataWithinIntervalRequest, O: GetMarketPriceDataWithinIntervalResponse }
]);
