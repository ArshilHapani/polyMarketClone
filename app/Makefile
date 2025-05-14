get-proto-files:
	@cp -r ../service-api/proto/*.proto ./public/proto/
	@echo "Proto files copied to public/proto directory."

generate-types: get-proto-files
	@echo "Generating types make sure that all files are in public/proto directory"
	@protoc -I=./public/proto \
			-I=$(shell pkg-config --variable=prefix protobuf)/include \
			--plugin=protoc-gen-ts=./node_modules/.bin/protoc-gen-ts \
			--ts_out=./src/generated \
			--ts_opt=generate_dependencies,long_type_number \
			$(shell find ./public/proto -name '*.proto')
	@echo "Types generated successfully."