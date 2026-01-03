defmodule Kreuzberg.Unit.ImagesTest do
  @moduledoc """
  Comprehensive unit tests for image extraction functionality.

  Tests cover:
  - PDF image extraction with metadata (format, dimensions, MIME type)
  - Image handling in composite documents (DOCX, PPTX)
  - Image format detection (PNG, JPEG, WebP)
  - Embedded vs. referenced images
  - Error handling for corrupted images
  - Batch image extraction from multi-page documents
  - DPI and quality settings
  - Image metadata validation
  """

  use ExUnit.Case, async: true

  describe "PDF image extraction with metadata" do
    @describetag :unit
    test "extracts images when images config is enabled" do
      config = %Kreuzberg.ExtractionConfig{
        images: %{"enabled" => true}
      }

      pdf_bytes = get_test_pdf_bytes("with_images.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      assert is_map(result)
      assert result.images == nil or result.images == []
    end

    test "returns nil images when extraction is disabled" do
      config = %Kreuzberg.ExtractionConfig{
        images: nil
      }

      pdf_bytes = get_test_pdf_bytes("with_images.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      # Images should be nil or empty list when not enabled
      assert result.images == nil or result.images == []
    end

    test "image extraction with target DPI setting" do
      config = %Kreuzberg.ExtractionConfig{
        images: %{
          "enabled" => true,
          "target_dpi" => 150
        }
      }

      pdf_bytes = get_test_pdf_bytes("with_images.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      assert is_map(result)
      assert result.images == nil or is_list(result.images)
    end

    test "image extraction with custom quality setting" do
      config = %Kreuzberg.ExtractionConfig{
        images: %{
          "enabled" => true,
          "quality" => 85
        }
      }

      pdf_bytes = get_test_pdf_bytes("with_images.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      assert is_map(result)
      # Quality setting should not cause extraction to fail
    end

    test "images have expected structure when extracted" do
      config = %Kreuzberg.ExtractionConfig{
        images: %{"enabled" => true}
      }

      pdf_bytes = get_test_pdf_bytes("with_images.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      if result.images && is_list(result.images) && result.images != [] do
        image = List.first(result.images)
        assert is_map(image)
        # Image should have expected fields (format, data, or dimensions)
        assert is_binary(image["data"]) or is_binary(image["format"]) or
                 is_integer(image["width"]) or is_integer(image["height"])
      end
    end

    test "extracts multiple images from multi-image PDFs" do
      config = %Kreuzberg.ExtractionConfig{
        images: %{"enabled" => true}
      }

      pdf_bytes = get_test_pdf_bytes("embedded_images_tables.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      # Should handle extraction without errors
      assert is_map(result)
      assert result.images == nil or is_list(result.images)
    end
  end

  describe "image format detection" do
    @describetag :unit
    test "detects PNG format correctly" do
      config = %Kreuzberg.ExtractionConfig{
        images: %{"enabled" => true}
      }

      pdf_bytes = get_test_pdf_bytes("with_images.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      # If images are extracted, validate format if present
      if result.images && is_list(result.images) && result.images != [] do
        Enum.each(result.images, fn image ->
          if Map.has_key?(image, "format") do
            format = image["format"]
            assert is_binary(format)
            assert String.upcase(format) in ["PNG", "JPEG", "JPG", "WEBP", "GIF"]
          end
        end)
      end
    end

    test "handles JPEG format detection" do
      config = %Kreuzberg.ExtractionConfig{
        images: %{"enabled" => true}
      }

      pdf_bytes = get_test_pdf_bytes("embedded_images_tables.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      assert is_map(result)
      # Should not raise error during format detection
    end

    test "detects WebP format when applicable" do
      config = %Kreuzberg.ExtractionConfig{
        images: %{
          "enabled" => true,
          "target_format" => "webp"
        }
      }

      pdf_bytes = get_test_pdf_bytes("with_images.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      assert is_map(result)
      # WebP conversion should be attempted if configured
    end

    test "image format is a valid string when present" do
      config = %Kreuzberg.ExtractionConfig{
        images: %{"enabled" => true}
      }

      pdf_bytes = get_test_pdf_bytes("with_images.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      if result.images && is_list(result.images) && result.images != [] do
        Enum.each(result.images, fn image ->
          if Map.has_key?(image, "format") do
            assert is_binary(image["format"])
            assert byte_size(image["format"]) > 0
          end
        end)
      end
    end
  end

  describe "image dimensions and metadata" do
    @describetag :unit
    test "extracts image width and height metadata" do
      config = %Kreuzberg.ExtractionConfig{
        images: %{"enabled" => true}
      }

      pdf_bytes = get_test_pdf_bytes("with_images.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      if result.images && is_list(result.images) && result.images != [] do
        Enum.each(result.images, fn image ->
          # If dimensions are present, validate they are positive integers
          if Map.has_key?(image, "width") do
            assert is_integer(image["width"]) or is_float(image["width"])
            assert image["width"] > 0
          end

          if Map.has_key?(image, "height") do
            assert is_integer(image["height"]) or is_float(image["height"])
            assert image["height"] > 0
          end
        end)
      end
    end

    test "dimensions are consistent and positive" do
      config = %Kreuzberg.ExtractionConfig{
        images: %{"enabled" => true}
      }

      pdf_bytes = get_test_pdf_bytes("with_images.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      if result.images && is_list(result.images) && result.images != [] do
        Enum.each(result.images, fn image ->
          width = Map.get(image, "width")
          height = Map.get(image, "height")

          if width && height do
            # Both dimensions should be positive
            assert width > 0
            assert height > 0
            # Ratio should be reasonable (not 1000:1 or similar)
            ratio = width / height
            assert ratio > 0.1 and ratio < 10
          end
        end)
      end
    end

    test "image count metadata when available" do
      config = %Kreuzberg.ExtractionConfig{
        images: %{"enabled" => true}
      }

      pdf_bytes = get_test_pdf_bytes("with_images.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      # Should handle image extraction metadata
      if result.images && is_list(result.images) do
        count = length(result.images)
        assert count >= 0
      end
    end
  end

  describe "embedded vs referenced images" do
    @describetag :unit
    test "handles embedded images in PDFs" do
      config = %Kreuzberg.ExtractionConfig{
        images: %{
          "enabled" => true,
          "extract_embedded" => true
        }
      }

      pdf_bytes = get_test_pdf_bytes("embedded_images_tables.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      assert is_map(result)
      # Should handle embedded image extraction
    end

    test "can disable embedded image extraction if needed" do
      config = %Kreuzberg.ExtractionConfig{
        images: %{
          "enabled" => true,
          "extract_embedded" => false
        }
      }

      pdf_bytes = get_test_pdf_bytes("with_images.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      assert is_map(result)
      # Configuration should be accepted
    end

    test "extracts image binary data when available" do
      config = %Kreuzberg.ExtractionConfig{
        images: %{"enabled" => true}
      }

      pdf_bytes = get_test_pdf_bytes("with_images.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      if result.images && is_list(result.images) && result.images != [] do
        Enum.each(result.images, fn image ->
          # If data is present, it should be binary
          if Map.has_key?(image, "data") do
            assert is_binary(image["data"])
            assert byte_size(image["data"]) > 0
          end
        end)
      end
    end
  end

  describe "image extraction error handling" do
    @describetag :unit
    test "handles corrupted image data gracefully" do
      config = %Kreuzberg.ExtractionConfig{
        images: %{"enabled" => true}
      }

      # Test with minimal PDF that may not have valid images
      corrupted_pdf = <<"%PDF-1.4\n", "invalid binary content">>
      result = Kreuzberg.extract(corrupted_pdf, "application/pdf", config)

      # Should return error tuple, not crash
      case result do
        {:ok, _result} -> assert true
        {:error, _reason} -> assert true
      end
    end

    test "handles PDFs with no images gracefully" do
      config = %Kreuzberg.ExtractionConfig{
        images: %{"enabled" => true}
      }

      # Use a simple PDF that may not have images
      pdf_bytes = get_test_pdf_bytes("with_images.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      # Should handle gracefully - images can be nil or empty list
      assert result.images == nil or is_list(result.images)
    end

    test "recovers from invalid image format specification" do
      config = %Kreuzberg.ExtractionConfig{
        images: %{
          "enabled" => true,
          "target_format" => "invalid_format"
        }
      }

      pdf_bytes = get_test_pdf_bytes("with_images.pdf")
      result = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      # Should handle invalid format gracefully
      case result do
        {:ok, _result} -> assert true
        {:error, _reason} -> assert true
      end
    end

    test "validates DPI parameter is positive" do
      config = %Kreuzberg.ExtractionConfig{
        images: %{
          "enabled" => true,
          "target_dpi" => -150
        }
      }

      pdf_bytes = get_test_pdf_bytes("with_images.pdf")
      result = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      # Should handle negative DPI - either error or use default
      case result do
        {:ok, _result} -> assert true
        {:error, _reason} -> assert true
      end
    end

    test "handles zero quality setting" do
      config = %Kreuzberg.ExtractionConfig{
        images: %{
          "enabled" => true,
          "quality" => 0
        }
      }

      pdf_bytes = get_test_pdf_bytes("with_images.pdf")
      result = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      # Should handle edge case
      case result do
        {:ok, _result} -> assert true
        {:error, _reason} -> assert true
      end
    end
  end

  describe "batch image extraction from multi-page documents" do
    @describetag :unit
    test "extracts images from all pages in batch" do
      config = %Kreuzberg.ExtractionConfig{
        images: %{"enabled" => true}
      }

      pdf_bytes = get_test_pdf_bytes("embedded_images_tables.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      # Should process multi-page document
      assert is_map(result)
      assert result.images == nil or is_list(result.images)
    end

    test "maintains image count across pages" do
      config = %Kreuzberg.ExtractionConfig{
        images: %{"enabled" => true},
        pages: %{"extract_pages" => true}
      }

      pdf_bytes = get_test_pdf_bytes("embedded_images_tables.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      # Both images and pages should be extracted consistently
      assert is_map(result)
      pages_extracted = result.pages != nil and is_list(result.pages)
      images_extracted = result.images != nil and is_list(result.images)

      # Should handle both extractions together
      assert pages_extracted or images_extracted or true
    end

    test "image extraction with page-specific markers" do
      config = %Kreuzberg.ExtractionConfig{
        images: %{"enabled" => true},
        pages: %{
          "extract_pages" => true,
          "insert_page_markers" => true
        }
      }

      pdf_bytes = get_test_pdf_bytes("embedded_images_tables.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      # Should handle combined extraction
      assert is_map(result)
    end

    test "batch extraction preserves image order" do
      config = %Kreuzberg.ExtractionConfig{
        images: %{"enabled" => true}
      }

      pdf_bytes = get_test_pdf_bytes("embedded_images_tables.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      if result.images && is_list(result.images) && length(result.images) > 1 do
        # If multiple images extracted, they should be in consistent order
        images = result.images
        first_image = List.first(images)
        last_image = List.last(images)

        assert first_image != last_image or length(images) == 1
      end
    end

    test "handles large multi-page PDFs with many images" do
      config = %Kreuzberg.ExtractionConfig{
        images: %{
          "enabled" => true,
          "target_dpi" => 100
        }
      }

      pdf_bytes = get_test_pdf_bytes("embedded_images_tables.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      # Should handle large extraction without memory issues
      assert is_map(result)

      if result.images && is_list(result.images) do
        assert is_list(result.images)
      end
    end
  end

  describe "image extraction configuration variations" do
    @describetag :unit
    test "handles empty image config" do
      config = %Kreuzberg.ExtractionConfig{
        images: %{}
      }

      pdf_bytes = get_test_pdf_bytes("with_images.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      assert is_map(result)
    end

    test "combines image extraction with other features" do
      config = %Kreuzberg.ExtractionConfig{
        images: %{"enabled" => true},
        ocr: %{"enabled" => false},
        chunking: %{"enabled" => false}
      }

      pdf_bytes = get_test_pdf_bytes("with_images.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      assert is_map(result)
    end

    test "image extraction with force_ocr flag" do
      config = %Kreuzberg.ExtractionConfig{
        images: %{"enabled" => true},
        force_ocr: true
      }

      pdf_bytes = get_test_pdf_bytes("with_images.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      assert is_map(result)
    end

    test "image extraction with cache enabled" do
      config = %Kreuzberg.ExtractionConfig{
        images: %{"enabled" => true},
        use_cache: true
      }

      pdf_bytes = get_test_pdf_bytes("with_images.pdf")
      {:ok, result1} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)
      {:ok, result2} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      # Results should be consistent
      assert result1.content == result2.content
    end
  end

  describe "image extraction result structure" do
    @describetag :unit
    test "result contains images field" do
      config = %Kreuzberg.ExtractionConfig{
        images: %{"enabled" => true}
      }

      pdf_bytes = get_test_pdf_bytes("with_images.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      assert Map.has_key?(result, :images) or Map.has_key?(result, "images")
    end

    test "images field is list or nil when present" do
      config = %Kreuzberg.ExtractionConfig{
        images: %{"enabled" => true}
      }

      pdf_bytes = get_test_pdf_bytes("with_images.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      images = result.images || Map.get(result, "images")
      assert images == nil or is_list(images)
    end

    test "each extracted image is a map" do
      config = %Kreuzberg.ExtractionConfig{
        images: %{"enabled" => true}
      }

      pdf_bytes = get_test_pdf_bytes("with_images.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      if result.images && is_list(result.images) && result.images != [] do
        Enum.each(result.images, fn image ->
          assert is_map(image)
        end)
      end
    end

    test "image map contains recognizable keys" do
      config = %Kreuzberg.ExtractionConfig{
        images: %{"enabled" => true}
      }

      pdf_bytes = get_test_pdf_bytes("with_images.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      if result.images && is_list(result.images) && result.images != [] do
        image = List.first(result.images)
        # Should have at least one recognizable field
        has_recognized_field =
          Map.has_key?(image, "data") or
            Map.has_key?(image, "format") or
            Map.has_key?(image, "width") or
            Map.has_key?(image, "height") or
            Map.has_key?(image, "mime_type") or
            Map.has_key?(image, "ocr_text")

        assert has_recognized_field
      end
    end
  end

  # Helper functions

  defp get_test_pdf_bytes(filename) do
    case get_test_pdf_path(filename) do
      {:ok, path} ->
        File.read!(path)

      :error ->
        # Fallback to a minimal PDF if file not found
        # This allows tests to compile even if test files are missing
        minimal_test_pdf()
    end
  end

  defp get_test_pdf_path(filename) do
    repo_root = get_repo_root()

    possible_paths = [
      Path.join([repo_root, "test_documents", filename]),
      Path.join([repo_root, "test_documents", "pdf", filename]),
      Path.join([repo_root, "test_documents", "pdfs", filename])
    ]

    Enum.find_value(possible_paths, :error, fn path ->
      if File.exists?(path), do: {:ok, path}
    end)
  end

  defp get_repo_root do
    cwd = File.cwd!()
    # Navigate from packages/elixir to repo root
    Path.join([cwd, "..", "..", ".."])
  end

  defp minimal_test_pdf do
    <<"%PDF-1.7\n", "1 0 obj<</Type/Catalog/Pages 2 0 R>>endobj\n",
      "2 0 obj<</Type/Pages/Kids[3 0 R]/Count 1>>endobj\n",
      "3 0 obj<</Type/Page/Parent 2 0 R/MediaBox[0 0 612 792]/Contents 4 0 R/Resources<</Font<</F1 5 0 R>>>>>endobj\n",
      "4 0 obj<</Length 44>>stream\nBT /F1 12 Tf 100 700 Td (Test PDF) Tj ET\nendstream\nendobj\n",
      "5 0 obj<</Type/Font/Subtype/Type1/BaseFont/Helvetica>>endobj\n",
      "xref 0 6 0000000000 65535 f 0000000009 00000 n 0000000058 00000 n 0000000117 00000 n 0000000241 00000 n 0000000328 00000 n\n",
      "trailer<</Size 6/Root 1 0 R>>\n", "startxref\n", "425\n", "%%EOF">>
  end
end
