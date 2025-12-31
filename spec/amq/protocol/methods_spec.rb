# frozen_string_literal: true

RSpec.describe "AMQ::Protocol method classes" do
  describe AMQ::Protocol::Connection do
    describe "::StartOk" do
      it "encodes client properties" do
        result = AMQ::Protocol::Connection::StartOk.encode(
          { product: "test" },
          "PLAIN",
          "\x00guest\x00guest",
          "en_US"
        )

        expect(result).to be_a(String)
        expect(result.encoding).to eq(Encoding::BINARY)
      end
    end

    describe "::TuneOk" do
      it "encodes tune parameters" do
        result = AMQ::Protocol::Connection::TuneOk.encode(2047, 131072, 60)

        expect(result).to be_a(String)
        # Check method header
        expect(result[0, 4].unpack("nn")).to eq([10, 31])
      end
    end

    describe "::Open" do
      it "encodes virtual host" do
        result = AMQ::Protocol::Connection::Open.encode("/")

        expect(result).to be_a(String)
        expect(result[0, 4].unpack("nn")).to eq([10, 40])
      end
    end

    describe "::Close" do
      it "encodes close parameters" do
        result = AMQ::Protocol::Connection::Close.encode(200, "Normal shutdown", 0, 0)

        expect(result).to be_a(String)
        expect(result[0, 4].unpack("nn")).to eq([10, 50])
      end
    end

    describe "::CloseOk" do
      it "encodes empty close-ok" do
        result = AMQ::Protocol::Connection::CloseOk.encode

        expect(result).to be_a(String)
        expect(result.unpack("nn")).to eq([10, 51])
      end
    end
  end

  describe AMQ::Protocol::Channel do
    describe "::Open" do
      it "encodes channel open" do
        result = AMQ::Protocol::Channel::Open.encode("")

        expect(result).to be_a(String)
        expect(result[0, 4].unpack("nn")).to eq([20, 10])
      end
    end

    describe "::Close" do
      it "encodes channel close" do
        result = AMQ::Protocol::Channel::Close.encode(200, "Normal", 0, 0)

        expect(result).to be_a(String)
        expect(result[0, 4].unpack("nn")).to eq([20, 40])
      end
    end

    describe "::CloseOk" do
      it "encodes channel close-ok" do
        result = AMQ::Protocol::Channel::CloseOk.encode

        expect(result).to be_a(String)
        expect(result.unpack("nn")).to eq([20, 41])
      end
    end
  end

  describe AMQ::Protocol::Exchange do
    describe "::Declare" do
      it "encodes exchange declare" do
        result = AMQ::Protocol::Exchange::Declare.encode(
          "my-exchange",
          "direct",
          false,  # passive
          true,   # durable
          false,  # auto_delete
          false,  # internal
          false,  # nowait
          {}      # arguments
        )

        expect(result).to be_a(String)
        expect(result[0, 4].unpack("nn")).to eq([40, 10])
      end
    end

    describe "::Delete" do
      it "encodes exchange delete" do
        result = AMQ::Protocol::Exchange::Delete.encode("my-exchange", false, false)

        expect(result).to be_a(String)
        expect(result[0, 4].unpack("nn")).to eq([40, 20])
      end
    end
  end

  describe AMQ::Protocol::Queue do
    describe "::Declare" do
      it "encodes queue declare" do
        result = AMQ::Protocol::Queue::Declare.encode(
          "my-queue",
          false,  # passive
          true,   # durable
          false,  # exclusive
          false,  # auto_delete
          false,  # nowait
          { "x-message-ttl" => 60000 }
        )

        expect(result).to be_a(String)
        expect(result[0, 4].unpack("nn")).to eq([50, 10])
      end
    end

    describe "::Bind" do
      it "encodes queue bind" do
        result = AMQ::Protocol::Queue::Bind.encode(
          "my-queue",
          "my-exchange",
          "routing.key",
          false,
          {}
        )

        expect(result).to be_a(String)
        expect(result[0, 4].unpack("nn")).to eq([50, 20])
      end
    end

    describe "::Delete" do
      it "encodes queue delete" do
        result = AMQ::Protocol::Queue::Delete.encode("my-queue", false, false, false)

        expect(result).to be_a(String)
        expect(result[0, 4].unpack("nn")).to eq([50, 40])
      end
    end
  end

  describe AMQ::Protocol::Basic do
    describe "::Qos" do
      it "encodes qos" do
        result = AMQ::Protocol::Basic::Qos.encode(0, 10, false)

        expect(result).to be_a(String)
        expect(result[0, 4].unpack("nn")).to eq([60, 10])
      end
    end

    describe "::Consume" do
      it "encodes consume" do
        result = AMQ::Protocol::Basic::Consume.encode(
          "my-queue",
          "my-consumer",
          false,  # no_local
          true,   # no_ack
          false,  # exclusive
          false,  # nowait
          {}      # arguments
        )

        expect(result).to be_a(String)
        expect(result[0, 4].unpack("nn")).to eq([60, 20])
      end
    end

    describe "::Publish" do
      it "encodes publish" do
        result = AMQ::Protocol::Basic::Publish.encode(
          "my-exchange",
          "routing.key",
          false,  # mandatory
          false   # immediate
        )

        expect(result).to be_a(String)
        expect(result[0, 4].unpack("nn")).to eq([60, 40])
      end
    end

    describe "::Ack" do
      it "encodes ack" do
        result = AMQ::Protocol::Basic::Ack.encode(1, false)

        expect(result).to be_a(String)
        expect(result[0, 4].unpack("nn")).to eq([60, 80])
      end
    end

    describe "::Nack" do
      it "encodes nack" do
        result = AMQ::Protocol::Basic::Nack.encode(1, false, true)

        expect(result).to be_a(String)
        expect(result[0, 4].unpack("nn")).to eq([60, 120])
      end
    end

    describe "::Reject" do
      it "encodes reject" do
        result = AMQ::Protocol::Basic::Reject.encode(1, true)

        expect(result).to be_a(String)
        expect(result[0, 4].unpack("nn")).to eq([60, 90])
      end
    end
  end

  describe AMQ::Protocol::Tx do
    describe "::Select" do
      it "encodes tx.select" do
        result = AMQ::Protocol::Tx::Select.encode

        expect(result).to be_a(String)
        expect(result.unpack("nn")).to eq([90, 10])
      end
    end

    describe "::Commit" do
      it "encodes tx.commit" do
        result = AMQ::Protocol::Tx::Commit.encode

        expect(result).to be_a(String)
        expect(result.unpack("nn")).to eq([90, 20])
      end
    end

    describe "::Rollback" do
      it "encodes tx.rollback" do
        result = AMQ::Protocol::Tx::Rollback.encode

        expect(result).to be_a(String)
        expect(result.unpack("nn")).to eq([90, 30])
      end
    end
  end

  describe AMQ::Protocol::Confirm do
    describe "::Select" do
      it "encodes confirm.select" do
        result = AMQ::Protocol::Confirm::Select.encode(false)

        expect(result).to be_a(String)
        expect(result[0, 4].unpack("nn")).to eq([85, 10])
      end
    end

    describe "::SelectOk" do
      it "encodes confirm.select-ok" do
        result = AMQ::Protocol::Confirm::SelectOk.encode

        expect(result).to be_a(String)
        expect(result.unpack("nn")).to eq([85, 11])
      end
    end
  end
end
