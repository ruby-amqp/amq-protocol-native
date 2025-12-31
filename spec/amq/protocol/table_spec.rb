# frozen_string_literal: true

RSpec.describe AMQ::Protocol::Table do
  describe ".encode" do
    it "encodes an empty hash" do
      result = described_class.encode({})
      expect(result).to eq("\x00\x00\x00\x00".b)
    end

    it "encodes a string value" do
      result = described_class.encode({ "key" => "value" })
      expect(result).to be_a(String)
      expect(result.encoding).to eq(Encoding::BINARY)
    end

    it "encodes an integer value" do
      result = described_class.encode({ "count" => 42 })
      expect(result).to be_a(String)
    end

    it "encodes a boolean value" do
      result = described_class.encode({ "enabled" => true })
      expect(result).to be_a(String)
    end

    it "encodes a nil value" do
      result = described_class.encode({ "nothing" => nil })
      expect(result).to be_a(String)
    end

    it "encodes nested hashes" do
      result = described_class.encode({ "outer" => { "inner" => "value" } })
      expect(result).to be_a(String)
    end

    it "encodes arrays" do
      result = described_class.encode({ "items" => [1, 2, 3] })
      expect(result).to be_a(String)
    end

    it "encodes symbol keys" do
      result = described_class.encode({ key: "value" })
      expect(result).to be_a(String)
    end

    it "encodes symbol values" do
      result = described_class.encode({ "key" => :symbol_value })
      expect(result).to be_a(String)
    end

    it "encodes float values" do
      result = described_class.encode({ "pi" => 3.14159 })
      expect(result).to be_a(String)
    end

    it "encodes Time values" do
      result = described_class.encode({ "timestamp" => Time.at(1234567890) })
      expect(result).to be_a(String)
    end
  end

  describe ".decode" do
    it "decodes an empty table" do
      encoded = "\x00\x00\x00\x00".b
      result = described_class.decode(encoded)
      expect(result).to eq({})
    end

    it "round-trips a complex table" do
      original = {
        "string" => "hello",
        "integer" => 42,
        "boolean" => true,
        "float" => 3.14,
        "array" => [1, "two", false],
        "nested" => { "key" => "value" }
      }

      encoded = described_class.encode(original)
      decoded = described_class.decode(encoded)

      expect(decoded["string"]).to eq("hello")
      expect(decoded["integer"]).to eq(42)
      expect(decoded["boolean"]).to eq(true)
      expect(decoded["float"]).to be_within(0.001).of(3.14)
      expect(decoded["array"]).to eq([1, "two", false])
      expect(decoded["nested"]).to eq({ "key" => "value" })
    end

    it "round-trips a Time value" do
      timestamp = Time.at(1234567890)
      original = { "timestamp" => timestamp }

      encoded = described_class.encode(original)
      decoded = described_class.decode(encoded)

      expect(decoded["timestamp"]).to be_a(Time)
      expect(decoded["timestamp"].to_i).to eq(timestamp.to_i)
    end
  end

  describe ".length" do
    it "returns the length from encoded data" do
      encoded = described_class.encode({ "key" => "value" })
      length = described_class.length(encoded)
      expect(length).to be_a(Integer)
      expect(length).to be >= 0
    end
  end
end
