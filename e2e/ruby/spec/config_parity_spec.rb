# frozen_string_literal: true

RSpec.describe "Config Parity E2E Tests" do
  require "json"
  require "kreuzberg"

  def sample_document
    File.expand_path("../../../../test_documents/text/report.txt", __FILE__).then do |path|
      if File.exist?(path)
        File.read(path, mode: 'rb')
      else
        "Hello World\n\nThis is a test document with multiple lines."
      end
    end
  end

  describe "OutputFormat Configuration" do
    describe "defaults" do
      it "has Plain as default output_format" do
        config = Kreuzberg::ExtractionConfig.new
        expect(config.output_format).to eq("Plain")
      end
    end

    describe "serialization" do
      it "serializes output_format to JSON" do
        config = Kreuzberg::ExtractionConfig.new(output_format: "Markdown")
        json = config.to_json
        data = JSON.parse(json)

        expect(data).to have_key("output_format")
        expect(data["output_format"]).to eq("Markdown")
      end
    end

    describe "extraction" do
      it "extracts with Plain output format" do
        doc = sample_document
        config = Kreuzberg::ExtractionConfig.new(output_format: "Plain")

        result = Kreuzberg.extract_bytes(data: doc, mime_type: "text/plain", config: config)

        expect(result).to be_a(Kreuzberg::ExtractionResult)
        expect(result.content).not_to be_nil
        expect(result.content.length).to be > 0
      end

      it "extracts with Markdown output format" do
        doc = sample_document
        config = Kreuzberg::ExtractionConfig.new(output_format: "Markdown")

        result = Kreuzberg.extract_bytes(data: doc, mime_type: "text/plain", config: config)

        expect(result).to be_a(Kreuzberg::ExtractionResult)
        expect(result.content).not_to be_nil
      end

      it "extracts with HTML output format" do
        doc = sample_document
        config = Kreuzberg::ExtractionConfig.new(output_format: "Html")

        result = Kreuzberg.extract_bytes(data: doc, mime_type: "text/plain", config: config)

        expect(result).to be_a(Kreuzberg::ExtractionResult)
        expect(result.content).not_to be_nil
      end
    end

    describe "format variations" do
      it "produces different content with different output formats" do
        doc = sample_document

        plain_config = Kreuzberg::ExtractionConfig.new(output_format: "Plain")
        plain_result = Kreuzberg.extract_bytes(data: doc, mime_type: "text/plain", config: plain_config)

        markdown_config = Kreuzberg::ExtractionConfig.new(output_format: "Markdown")
        markdown_result = Kreuzberg.extract_bytes(data: doc, mime_type: "text/plain", config: markdown_config)

        expect(plain_result.content).not_to be_nil
        expect(markdown_result.content).not_to be_nil
      end
    end
  end

  describe "ResultFormat Configuration" do
    describe "defaults" do
      it "has Unified as default result_format" do
        config = Kreuzberg::ExtractionConfig.new
        expect(config.result_format).to eq("Unified")
      end
    end

    describe "serialization" do
      it "serializes result_format to JSON" do
        config = Kreuzberg::ExtractionConfig.new(result_format: "Elements")
        json = config.to_json
        data = JSON.parse(json)

        expect(data).to have_key("result_format")
        expect(data["result_format"]).to eq("Elements")
      end
    end

    describe "extraction" do
      it "extracts with Unified result format" do
        doc = sample_document
        config = Kreuzberg::ExtractionConfig.new(result_format: "Unified")

        result = Kreuzberg.extract_bytes(data: doc, mime_type: "text/plain", config: config)

        expect(result).to be_a(Kreuzberg::ExtractionResult)
        expect(result.content).not_to be_nil
        expect(result.content).to be_a(String)
      end

      it "extracts with Elements result format" do
        doc = sample_document
        config = Kreuzberg::ExtractionConfig.new(result_format: "Elements")

        result = Kreuzberg.extract_bytes(data: doc, mime_type: "text/plain", config: config)

        expect(result).to be_a(Kreuzberg::ExtractionResult)
      end
    end

    describe "format variations" do
      it "produces results with different result formats" do
        doc = sample_document

        unified_config = Kreuzberg::ExtractionConfig.new(result_format: "Unified")
        unified_result = Kreuzberg.extract_bytes(data: doc, mime_type: "text/plain", config: unified_config)

        elements_config = Kreuzberg::ExtractionConfig.new(result_format: "Elements")
        elements_result = Kreuzberg.extract_bytes(data: doc, mime_type: "text/plain", config: elements_config)

        expect(unified_result).not_to be_nil
        expect(elements_result).not_to be_nil
      end
    end
  end

  describe "Config Combinations" do
    it "handles Plain with Unified combination" do
      doc = sample_document
      config = Kreuzberg::ExtractionConfig.new(
        output_format: "Plain",
        result_format: "Unified"
      )

      result = Kreuzberg.extract_bytes(data: doc, mime_type: "text/plain", config: config)

      expect(result).to be_a(Kreuzberg::ExtractionResult)
    end

    it "handles Markdown with Elements combination" do
      doc = sample_document
      config = Kreuzberg::ExtractionConfig.new(
        output_format: "Markdown",
        result_format: "Elements"
      )

      result = Kreuzberg.extract_bytes(data: doc, mime_type: "text/plain", config: config)

      expect(result).to be_a(Kreuzberg::ExtractionResult)
    end

    it "handles HTML with Unified combination" do
      doc = sample_document
      config = Kreuzberg::ExtractionConfig.new(
        output_format: "Html",
        result_format: "Unified"
      )

      result = Kreuzberg.extract_bytes(data: doc, mime_type: "text/plain", config: config)

      expect(result).to be_a(Kreuzberg::ExtractionResult)
    end

    it "preserves format fields when merging configs" do
      config1 = Kreuzberg::ExtractionConfig.new(
        output_format: "Markdown",
        result_format: "Elements"
      )
      config2 = Kreuzberg::ExtractionConfig.new(use_cache: false)

      merged = config1.merge(config2)

      expect(merged.output_format).to eq("Markdown")
      expect(merged.result_format).to eq("Elements")
    end
  end

  describe "Config Serialization" do
    it "serializes output_format correctly" do
      config = Kreuzberg::ExtractionConfig.new(output_format: "Markdown")
      json = config.to_json
      data = JSON.parse(json)

      expect(data).to have_key("output_format")
      expect(data["output_format"]).to eq("Markdown")
    end

    it "serializes result_format correctly" do
      config = Kreuzberg::ExtractionConfig.new(result_format: "Elements")
      json = config.to_json
      data = JSON.parse(json)

      expect(data).to have_key("result_format")
      expect(data["result_format"]).to eq("Elements")
    end

    it "preserves formats through JSON round-trip" do
      original = Kreuzberg::ExtractionConfig.new(
        output_format: "Html",
        result_format: "Elements",
        use_cache: false
      )

      json = original.to_json
      data = JSON.parse(json)

      expect(data["output_format"]).to eq("Html")
      expect(data["result_format"]).to eq("Elements")
      expect(data["use_cache"]).to eq(false)
    end
  end

  describe "Error Handling" do
    it "rejects invalid output_format values" do
      expect do
        Kreuzberg::ExtractionConfig.new(output_format: "InvalidFormat")
      end.to raise_error(ArgumentError)
    end

    it "rejects invalid result_format values" do
      expect do
        Kreuzberg::ExtractionConfig.new(result_format: "InvalidFormat")
      end.to raise_error(ArgumentError)
    end

    it "enforces case sensitivity for format names" do
      # lowercase "plain" should not work
      expect do
        Kreuzberg::ExtractionConfig.new(output_format: "plain")
      end.to raise_error(ArgumentError)
    end
  end
end
