# frozen_string_literal: true

RSpec.describe AMQ::Protocol::Frame do
  describe "constants" do
    it "defines TYPES" do
      expect(described_class::TYPES).to eq({
        method: 1,
        headers: 2,
        body: 3,
        heartbeat: 8
      })
    end

    it "defines TYPES_REVERSE" do
      expect(described_class::TYPES_REVERSE).to eq({
        1 => :method,
        2 => :headers,
        3 => :body,
        8 => :heartbeat
      })
    end

    it "defines FINAL_OCTET" do
      expect(described_class::FINAL_OCTET.bytes).to eq([0xCE])
    end
  end

  describe ".encode" do
    it "encodes a method frame" do
      payload = "test payload"
      result = described_class.encode(:method, payload, 1)

      expect(result).to be_a(String)
      expect(result.encoding).to eq(Encoding::BINARY)
      expect(result[-1].ord).to eq(0xCE)
    end

    it "encodes a heartbeat frame" do
      result = described_class.encode(:heartbeat, "", 0)

      expect(result).to be_a(String)
      expect(result[0].ord).to eq(8) # heartbeat type
    end

    it "raises error for invalid channel" do
      expect {
        described_class.encode(:method, "payload", -1)
      }.to raise_error(RuntimeError)

      expect {
        described_class.encode(:method, "payload", 65536)
      }.to raise_error(RuntimeError)
    end
  end

  describe ".encode_to_array" do
    it "returns an array of components" do
      payload = "test"
      result = described_class.encode_to_array(:method, payload, 1)

      expect(result).to be_a(Array)
      expect(result.length).to eq(3)
      expect(result[1]).to eq(payload)
      expect(result[2].bytes).to eq([0xCE])
    end
  end

  describe ".decode_header" do
    it "decodes a frame header" do
      # Create a valid frame header: type=1 (method), channel=1, size=4
      # Pack format: C=type (1 byte), n=channel (2 bytes big-endian), N=size (4 bytes big-endian)
      header = [1, 1, 4].pack("CnN")

      type, channel, size = described_class.decode_header(header)

      expect(type).to eq(:method)
      expect(channel).to eq(1)
      expect(size).to eq(4)
    end

    it "raises error for empty header" do
      expect {
        described_class.decode_header("")
      }.to raise_error(RuntimeError)
    end
  end
end

RSpec.describe AMQ::Protocol::MethodFrame do
  it "has ID constant" do
    expect(described_class.id).to eq(1)
  end
end

RSpec.describe AMQ::Protocol::HeaderFrame do
  it "has ID constant" do
    expect(described_class.id).to eq(2)
  end
end

RSpec.describe AMQ::Protocol::BodyFrame do
  it "has ID constant" do
    expect(described_class.id).to eq(3)
  end
end

RSpec.describe AMQ::Protocol::HeartbeatFrame do
  it "has ID constant" do
    expect(described_class.id).to eq(8)
  end

  describe ".encode" do
    it "encodes a heartbeat frame" do
      result = described_class.encode
      expect(result).to be_a(String)
    end
  end
end
