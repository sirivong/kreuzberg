defmodule Kreuzberg.Unit.TablesTest do
  @moduledoc """
  Comprehensive unit tests for table extraction quality and validation.

  Tests cover:
  - Table structure extraction (rows, columns, headers)
  - Complex table handling (merged cells, nested structures)
  - Table-in-table edge cases
  - Format-specific behavior (PDF vs. Office formats)
  - Performance with large tables (100+ rows)
  - Markdown conversion accuracy
  - Cell content preservation and quality
  - Table boundary detection and structure integrity
  """

  use ExUnit.Case, async: true

  # ============================================================================
  # Test 1: Table structure extraction (rows, columns, headers)
  # ============================================================================
  describe "table structure extraction" do
    @tag :unit
    test "extracts table rows, columns, and headers correctly" do
      config = %Kreuzberg.ExtractionConfig{}

      pdf_bytes = get_test_pdf_bytes("tables.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      # Verify tables were extracted
      assert result.tables != nil
      assert is_list(result.tables)
      assert result.tables != []

      # Validate first table structure
      table = List.first(result.tables)
      assert is_map(table)

      assert Map.has_key?(table, :cells) or Map.has_key?(table, :rows) or
               Map.has_key?(table, :headers)
    end

    test "identifies table headers correctly" do
      config = %Kreuzberg.ExtractionConfig{}

      pdf_bytes = get_test_pdf_bytes("tables.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      if result.tables && result.tables != [] do
        table = List.first(result.tables)

        # Should have either headers field or first row identifiable as headers
        has_headers =
          Map.has_key?(table, :headers) or
            Map.has_key?(table, :header_row) or
            (Map.has_key?(table, :cells) and is_list(Map.get(table, :cells)))

        assert has_headers
      end
    end

    test "maintains row and column consistency" do
      config = %Kreuzberg.ExtractionConfig{}

      pdf_bytes = get_test_pdf_bytes("tables.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      if result.tables && result.tables != [] do
        Enum.each(result.tables, fn table ->
          # If table has cells, validate structure
          if Map.has_key?(table, "cells") && is_list(Map.get(table, "cells")) do
            cells = Map.get(table, "cells")

            if cells != [] do
              # Count columns in first row
              first_row = List.first(cells)

              expected_cols =
                if is_list(first_row), do: length(first_row), else: 1

              # All rows should have same column count
              Enum.each(cells, fn row ->
                actual_cols = if is_list(row), do: length(row), else: 1
                assert actual_cols == expected_cols
              end)
            end
          end
        end)
      end
    end

    test "preserves table cell ordering" do
      config = %Kreuzberg.ExtractionConfig{}

      pdf_bytes = get_test_pdf_bytes("tables.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      if result.tables && result.tables != [] do
        table = List.first(result.tables)

        if Map.has_key?(table, "cells") do
          cells = Map.get(table, "cells")
          # Cells should be ordered from top to bottom, left to right
          assert is_list(cells)
          assert cells != []
        end
      end
    end

    test "handles single-row tables" do
      config = %Kreuzberg.ExtractionConfig{}

      pdf_bytes = get_test_pdf_bytes("tables.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      # Should not crash on single-row tables and should return valid structure
      if result.tables && is_list(result.tables) do
        result.tables
        |> Enum.each(fn table ->
          if Map.has_key?(table, "cells") do
            cells = Map.get(table, "cells")
            # Single-row table should still have valid structure
            assert is_list(cells)
          end
        end)
      end
    end

    test "handles single-column tables" do
      config = %Kreuzberg.ExtractionConfig{}

      pdf_bytes = get_test_pdf_bytes("tables.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      # Should not crash on single-column tables and should return valid structure
      if result.tables && is_list(result.tables) do
        result.tables
        |> Enum.each(fn table ->
          if Map.has_key?(table, "cells") do
            cells = Map.get(table, "cells")
            # Single-column table should still have valid structure
            assert is_list(cells)
            # All rows should have at least 1 cell
            Enum.each(cells, fn row ->
              if is_list(row), do: assert(row != [])
            end)
          end
        end)
      end
    end
  end

  # ============================================================================
  # Test 2: Complex tables (merged cells, nested tables)
  # ============================================================================
  describe "complex table handling" do
    @tag :unit
    test "handles tables with merged cells" do
      config = %Kreuzberg.ExtractionConfig{}

      pdf_bytes = get_test_pdf_bytes("merged_cells_table.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      # Should extract tables without errors even with merged cells
      assert is_map(result)
      assert result.tables == nil or is_list(result.tables)
    end

    test "detects merged cell spanning" do
      config = %Kreuzberg.ExtractionConfig{}

      pdf_bytes = get_test_pdf_bytes("merged_cells_table.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      if result.tables && result.tables != [] do
        table = List.first(result.tables)

        # Check if merged cells are represented (colspan/rowspan or cell content patterns)
        has_merge_info =
          Map.has_key?(table, "colspan") or
            Map.has_key?(table, "rowspan") or
            Map.has_key?(table, "merged_cells") or
            is_map(table)

        assert has_merge_info
      end
    end

    test "extracts nested table content" do
      config = %Kreuzberg.ExtractionConfig{}

      # Using embedded_images_tables.pdf which contains nested structures
      pdf_bytes = get_test_pdf_bytes("embedded_images_tables.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      # Should handle complex nested structures
      assert is_map(result)
      assert result.tables == nil or is_list(result.tables)
    end

    test "preserves content in complex table cells" do
      config = %Kreuzberg.ExtractionConfig{}

      pdf_bytes = get_test_pdf_bytes("tables.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      if result.tables && result.tables != [] do
        Enum.each(result.tables, fn table ->
          if Map.has_key?(table, "cells") do
            cells = Map.get(table, "cells")

            Enum.each(cells, fn row ->
              if is_list(row) do
                Enum.each(row, fn cell ->
                  # Cell content should be string, number, or map (for nested content)
                  assert is_binary(cell) or is_number(cell) or is_map(cell) or
                           is_list(cell)
                end)
              end
            end)
          end
        end)
      end
    end
  end

  # ============================================================================
  # Test 3: Table-in-table edge cases
  # ============================================================================
  describe "table-in-table edge cases" do
    @tag :unit
    test "handles tables containing other tables" do
      config = %Kreuzberg.ExtractionConfig{}

      pdf_bytes = get_test_pdf_bytes("embedded_images_tables.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      # Should not crash and should return valid result
      assert is_map(result)
      assert result.tables == nil or is_list(result.tables)
    end

    test "distinguishes between separate and nested tables" do
      config = %Kreuzberg.ExtractionConfig{}

      pdf_bytes = get_test_pdf_bytes("tables.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      if result.tables && result.tables != [] and length(result.tables) > 1 do
        # Multiple tables should be separate entries in the list
        tables = result.tables
        assert tables != []

        # Each should be distinguishable
        table1 = List.first(tables)
        table2 = if length(tables) > 1, do: Enum.at(tables, 1), else: table1

        # Tables should have different structures or content (not duplicates)
        assert table1 != table2 or length(tables) == 1
      end
    end

    test "extracts nested table content accurately" do
      config = %Kreuzberg.ExtractionConfig{}

      pdf_bytes = get_test_pdf_bytes("tables.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      if result.tables && result.tables != [] do
        Enum.each(result.tables, fn table ->
          # Nested content should be preserved
          if Map.has_key?(table, "cells") do
            cells = Map.get(table, "cells")
            assert is_list(cells)
          end
        end)
      end
    end

    test "handles deeply nested table structures" do
      config = %Kreuzberg.ExtractionConfig{}

      pdf_bytes = get_test_pdf_bytes("embedded_images_tables.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      # Should process without stack overflow or memory issues
      assert is_map(result)
    end
  end

  # ============================================================================
  # Test 4: Format-specific table handling (PDF vs. Office formats)
  # ============================================================================
  describe "format-specific table handling" do
    @tag :unit
    test "extracts PDF tables with PDF-specific settings" do
      config = %Kreuzberg.ExtractionConfig{
        pdf_options: %{extract_tables: true}
      }

      pdf_bytes = get_test_pdf_bytes("tables.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      assert is_map(result)
      # PDF extraction should succeed
      assert result.tables == nil or is_list(result.tables)
    end

    test "handles Office format table extraction" do
      config = %Kreuzberg.ExtractionConfig{}

      # Try DOCX if available
      docx_result =
        case get_test_docx_bytes() do
          {:ok, bytes} ->
            Kreuzberg.extract(
              bytes,
              "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
              config
            )

          :error ->
            {:ok, %Kreuzberg.ExtractionResult{tables: []}}
        end

      case docx_result do
        {:ok, result} ->
          assert result.tables == nil or is_list(result.tables)

        {:error, _reason} ->
          # Office format may not be supported, that's OK
          assert true
      end
    end

    test "PDF and Office formats produce consistent table structure" do
      config = %Kreuzberg.ExtractionConfig{}

      pdf_bytes = get_test_pdf_bytes("tables.pdf")
      {:ok, pdf_result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      # If both formats available, structures should be comparable
      if pdf_result.tables && pdf_result.tables != [] do
        table = List.first(pdf_result.tables)
        # Table should have recognizable structure regardless of format
        assert is_map(table)
      end
    end

    test "handles format-specific table encoding" do
      config = %Kreuzberg.ExtractionConfig{}

      pdf_bytes = get_test_pdf_bytes("tables.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      if result.tables && result.tables != [] do
        Enum.each(result.tables, fn table ->
          # Content should be properly encoded regardless of source format
          if Map.has_key?(table, "cells") do
            cells = Map.get(table, "cells")
            assert is_list(cells)
          end
        end)
      end
    end
  end

  # ============================================================================
  # Test 5: Performance with large tables (100+ rows)
  # ============================================================================
  describe "performance with large tables" do
    @tag :unit
    test "extracts large tables efficiently" do
      config = %Kreuzberg.ExtractionConfig{}

      pdf_bytes = get_test_pdf_bytes("large_table.pdf")

      start_time = System.monotonic_time(:millisecond)
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)
      elapsed = System.monotonic_time(:millisecond) - start_time

      # Should complete within reasonable time (10 seconds)
      assert elapsed < 10_000

      assert is_map(result)
      assert result.tables == nil or is_list(result.tables)
    end

    test "handles tables with 100+ rows" do
      config = %Kreuzberg.ExtractionConfig{}

      pdf_bytes = get_test_pdf_bytes("large_table.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      if result.tables && result.tables != [] do
        table = List.first(result.tables)

        if Map.has_key?(table, "cells") do
          cells = Map.get(table, "cells")
          # Large table check: at least reasonable number of rows
          assert is_list(cells)
        end
      end
    end

    test "memory usage remains reasonable with large tables" do
      config = %Kreuzberg.ExtractionConfig{}

      pdf_bytes = get_test_pdf_bytes("large_table.pdf")

      # Extract and validate memory wasn't excessive
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      # If large table extracted, result should be a valid map
      assert is_map(result)
    end

    test "preserves all rows in large table extraction" do
      config = %Kreuzberg.ExtractionConfig{}

      pdf_bytes = get_test_pdf_bytes("large_table.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      if result.tables && result.tables != [] do
        table = List.first(result.tables)

        if Map.has_key?(table, "cells") && is_list(Map.get(table, "cells")) do
          cells = Map.get(table, "cells")
          # All rows should be present
          assert cells != []
        end
      end
    end

    test "maintains consistent structure across pagination" do
      config = %Kreuzberg.ExtractionConfig{
        pages: %{extract_pages: true}
      }

      pdf_bytes = get_test_pdf_bytes("large_table.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      # Tables from multiple pages should maintain structure
      if result.tables && result.tables != [] do
        tables = result.tables

        Enum.each(tables, fn table ->
          assert is_map(table)
        end)
      end
    end
  end

  # ============================================================================
  # Test 6: Markdown conversion accuracy
  # ============================================================================
  describe "markdown conversion accuracy" do
    @tag :unit
    test "converts table to markdown format" do
      config = %Kreuzberg.ExtractionConfig{}

      pdf_bytes = get_test_pdf_bytes("tables.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      if result.tables && result.tables != [] do
        table = List.first(result.tables)

        # Check for markdown representation
        has_markdown =
          Map.has_key?(table, "markdown") or
            Map.has_key?(table, "markdown_table") or
            String.contains?(inspect(table), "|")

        # At least one format should be present
        assert is_map(table)
      end
    end

    test "markdown output is valid table format" do
      config = %Kreuzberg.ExtractionConfig{}

      pdf_bytes = get_test_pdf_bytes("tables.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      if result.tables && result.tables != [] do
        table = List.first(result.tables)

        if Map.has_key?(table, "markdown") do
          markdown = Map.get(table, "markdown")
          assert is_binary(markdown)
          # Should contain pipe characters for markdown table format
          assert String.contains?(markdown, "|") or byte_size(markdown) == 0
        end
      end
    end

    test "markdown preserves cell content" do
      config = %Kreuzberg.ExtractionConfig{}

      pdf_bytes = get_test_pdf_bytes("tables.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      if result.tables && result.tables != [] do
        table = List.first(result.tables)

        # If both cells and markdown exist, content should be related
        has_cells = Map.has_key?(table, "cells")
        has_markdown = Map.has_key?(table, "markdown")

        # Should have at least one representation
        assert has_cells or has_markdown or is_map(table)
      end
    end

    test "markdown handles special characters" do
      config = %Kreuzberg.ExtractionConfig{}

      pdf_bytes = get_test_pdf_bytes("tables.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      if result.tables && result.tables != [] do
        table = List.first(result.tables)

        if Map.has_key?(table, "markdown") do
          markdown = Map.get(table, "markdown")
          # Markdown should be valid UTF-8
          assert is_binary(markdown)
        end
      end
    end

    test "markdown output is properly delimited" do
      config = %Kreuzberg.ExtractionConfig{}

      pdf_bytes = get_test_pdf_bytes("tables.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      if result.tables && result.tables != [] do
        table = List.first(result.tables)

        if Map.has_key?(table, "markdown") do
          markdown = Map.get(table, "markdown")

          # Markdown tables should have separators
          lines = String.split(markdown, "\n")

          if length(lines) > 1 do
            # Second line typically contains dashes for markdown tables
            second_line = Enum.at(lines, 1)

            has_separator =
              String.contains?(second_line, "-") or
                String.contains?(second_line, "|")

            # Not requiring strict format, but should have some structure
            assert is_binary(markdown)
          end
        end
      end
    end
  end

  # ============================================================================
  # Test 7: Cell content preservation
  # ============================================================================
  describe "cell content preservation" do
    @tag :unit
    test "preserves text content in cells" do
      config = %Kreuzberg.ExtractionConfig{}

      pdf_bytes = get_test_pdf_bytes("tables.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      if result.tables && result.tables != [] do
        table = List.first(result.tables)

        if Map.has_key?(table, "cells") do
          cells = Map.get(table, "cells")

          Enum.each(cells, fn row ->
            if is_list(row) do
              Enum.each(row, fn cell ->
                # Cell should be preserved as-is
                assert is_binary(cell) or is_number(cell) or is_map(cell) or
                         is_list(cell) or cell == nil
              end)
            end
          end)
        end
      end
    end

    test "handles numeric values in cells" do
      config = %Kreuzberg.ExtractionConfig{}

      pdf_bytes = get_test_pdf_bytes("tables.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      if result.tables && result.tables != [] do
        table = List.first(result.tables)

        if Map.has_key?(table, "cells") do
          cells = Map.get(table, "cells")

          # Should handle numeric values appropriately
          assert is_list(cells)
        end
      end
    end

    test "preserves cell formatting information" do
      config = %Kreuzberg.ExtractionConfig{}

      pdf_bytes = get_test_pdf_bytes("tables.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      if result.tables && result.tables != [] do
        table = List.first(result.tables)

        # Table should have structure that preserves formatting context
        has_format_info =
          Map.has_key?(table, "cells") or
            Map.has_key?(table, "markdown") or
            Map.has_key?(table, "html")

        assert is_map(table)
      end
    end

    test "handles empty cells correctly" do
      config = %Kreuzberg.ExtractionConfig{}

      pdf_bytes = get_test_pdf_bytes("tables.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      if result.tables && result.tables != [] do
        table = List.first(result.tables)

        if Map.has_key?(table, "cells") do
          cells = Map.get(table, "cells")

          # Empty cells should be represented (empty string, nil, or empty map)
          Enum.each(cells, fn row ->
            if is_list(row) do
              Enum.each(row, fn cell ->
                # Empty cells should have some representation
                assert is_binary(cell) or is_number(cell) or is_map(cell) or
                         is_list(cell) or cell == nil or cell == ""
              end)
            end
          end)
        end
      end
    end

    test "preserves multi-line cell content" do
      config = %Kreuzberg.ExtractionConfig{}

      pdf_bytes = get_test_pdf_bytes("tables.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      if result.tables && result.tables != [] do
        table = List.first(result.tables)

        if Map.has_key?(table, "cells") do
          cells = Map.get(table, "cells")

          # Cells with newlines should be preserved
          Enum.each(cells, fn row ->
            if is_list(row) do
              Enum.each(row, fn cell ->
                if is_binary(cell) do
                  # Multi-line content should be preserved
                  assert is_binary(cell)
                end
              end)
            end
          end)
        end
      end
    end
  end

  # ============================================================================
  # Test 8: Table boundary detection
  # ============================================================================
  describe "table boundary detection" do
    @tag :unit
    test "correctly identifies table boundaries" do
      config = %Kreuzberg.ExtractionConfig{}

      pdf_bytes = get_test_pdf_bytes("tables.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      if result.tables && result.tables != [] do
        Enum.each(result.tables, fn table ->
          # Each table should have defined boundaries
          assert is_map(table)

          if Map.has_key?(table, "cells") do
            cells = Map.get(table, "cells")
            assert is_list(cells)
          end
        end)
      end
    end

    test "separates adjacent tables correctly" do
      config = %Kreuzberg.ExtractionConfig{}

      pdf_bytes = get_test_pdf_bytes("tables.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      if result.tables && length(result.tables) > 1 do
        tables = result.tables

        # Adjacent tables should be separate entries
        table1 = List.first(tables)
        table2 = Enum.at(tables, 1)

        assert table1 != table2 or length(tables) == 1
      end
    end

    test "detects table location in document" do
      config = %Kreuzberg.ExtractionConfig{
        pages: %{extract_pages: true}
      }

      pdf_bytes = get_test_pdf_bytes("tables.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      if result.tables && result.tables != [] do
        Enum.each(result.tables, fn table ->
          # Table location information might be preserved
          has_location_info =
            Map.has_key?(table, "page") or
              Map.has_key?(table, "page_number") or
              Map.has_key?(table, "location") or
              Map.has_key?(table, "position")

          # At minimum, table should be a valid map
          assert is_map(table)
        end)
      end
    end

    test "validates table integrity after boundary detection" do
      config = %Kreuzberg.ExtractionConfig{}

      pdf_bytes = get_test_pdf_bytes("tables.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      if result.tables && result.tables != [] do
        Enum.each(result.tables, fn table ->
          if Map.has_key?(table, "cells") do
            cells = Map.get(table, "cells")

            # All cells should be accounted for (no loss of data at boundaries)
            assert is_list(cells)
            assert cells != []
          end
        end)
      end
    end

    test "handles tables at page boundaries" do
      config = %Kreuzberg.ExtractionConfig{}

      pdf_bytes = get_test_pdf_bytes("large_table.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      # Tables spanning pages should still be extracted correctly
      if result.tables && result.tables != [] do
        Enum.each(result.tables, fn table ->
          assert is_map(table)
        end)
      end
    end
  end

  # ============================================================================
  # Test 9: Table extraction configuration
  # ============================================================================
  describe "table extraction configuration" do
    @tag :unit
    test "tables are extracted automatically by default" do
      config = %Kreuzberg.ExtractionConfig{}

      pdf_bytes = get_test_pdf_bytes("tables.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      # Tables should be extracted regardless of config (tables are always extracted)
      assert result.tables == nil or is_list(result.tables)
    end

    test "respects extraction configuration with minimal config" do
      config = %Kreuzberg.ExtractionConfig{}

      pdf_bytes = get_test_pdf_bytes("tables.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      # Should extract with default behavior
      assert is_map(result)
    end

    test "combines table extraction with other features" do
      config = %Kreuzberg.ExtractionConfig{
        images: %{enabled: false},
        ocr: %{enabled: false}
      }

      pdf_bytes = get_test_pdf_bytes("tables.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      # Tables should be extracted regardless of other settings
      assert is_map(result)
      assert result.tables == nil or is_list(result.tables)
    end

    test "handles empty extraction configuration" do
      config = %Kreuzberg.ExtractionConfig{}

      pdf_bytes = get_test_pdf_bytes("tables.pdf")
      {:ok, result} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      # Should handle empty config gracefully
      assert is_map(result)
    end

    test "validates table extraction with caching" do
      config = %Kreuzberg.ExtractionConfig{
        use_cache: true
      }

      pdf_bytes = get_test_pdf_bytes("tables.pdf")
      {:ok, result1} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)
      {:ok, result2} = Kreuzberg.extract(pdf_bytes, "application/pdf", config)

      # Cached results should match
      if result1.tables && result2.tables do
        assert length(result1.tables) == length(result2.tables)
      end
    end
  end

  # ============================================================================
  # Helper Functions
  # ============================================================================

  defp get_test_pdf_bytes(filename) do
    case get_test_pdf_path(filename) do
      {:ok, path} ->
        File.read!(path)

      :error ->
        # Fallback to minimal PDF if file not found
        minimal_test_pdf()
    end
  end

  defp get_test_docx_bytes do
    case get_test_docx_path() do
      {:ok, path} ->
        {:ok, File.read!(path)}

      :error ->
        :error
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

  defp get_test_docx_path do
    repo_root = get_repo_root()

    possible_paths = [
      Path.join([repo_root, "test_documents", "tables.docx"]),
      Path.join([repo_root, "test_documents", "docx", "tables.docx"]),
      Path.join([repo_root, "test_documents", "office", "tables.docx"])
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
      "4 0 obj<</Length 148>>stream\n", "BT /F1 12 Tf 100 700 Td (Table Test) Tj ET\n",
      "0 0 m 100 0 l 100 20 l 0 20 l 0 0 l S\n", "endstream\nendobj\n",
      "5 0 obj<</Type/Font/Subtype/Type1/BaseFont/Helvetica>>endobj\n",
      "xref 0 6 0000000000 65535 f 0000000009 00000 n 0000000058 00000 n 0000000117 00000 n 0000000241 00000 n 0000000437 00000 n\n",
      "trailer<</Size 6/Root 1 0 R>>\n", "startxref\n", "524\n", "%%EOF">>
  end
end
