# Auto-generated Elixir E2E test helpers
#
# To regenerate: cargo run -p kreuzberg-e2e-generator -- generate --lang elixir

defmodule E2E.Helpers do
  @moduledoc """
  Test helpers for E2E extraction tests.
  """

  import ExUnit.Assertions

  @workspace_root Path.expand("../../../..", __DIR__)
  @test_documents Path.join(@workspace_root, "test_documents")

  def resolve_document(relative) do
    Path.join(@test_documents, relative)
  end

  def build_config(nil), do: nil
  def build_config(raw) when is_map(raw) and map_size(raw) == 0, do: nil
  def build_config(raw) when is_map(raw) do
    atomize_keys(raw)
  end

  defp atomize_keys(value) when is_map(value) do
    Map.new(value, fn {key, val} ->
      atom_key = if is_binary(key), do: String.to_atom(key), else: key
      {atom_key, atomize_keys(val)}
    end)
  end

  defp atomize_keys(value) when is_list(value) do
    Enum.map(value, &atomize_keys/1)
  end

  defp atomize_keys(value), do: value

  def skip_reason_for(error, fixture_id, requirements, notes \\ nil) do
    message = Exception.message(error)
    downcased = String.downcase(message)
    requirement_hit = Enum.any?(requirements, fn req ->
      String.contains?(downcased, String.downcase(req))
    end)
    missing_dependency = String.contains?(downcased, "missing dependency")
    unsupported_format = String.contains?(downcased, "unsupported format")

    case {missing_dependency, unsupported_format, requirement_hit} do
      {true, _, _} ->
        reason = "missing dependency"
        details = "Skipping #{fixture_id}: #{reason}. #{inspect(error)}"
        details = if notes, do: "#{details} Notes: #{notes}", else: details
        IO.warn(details)
        details

      {_, true, _} ->
        reason = "unsupported format"
        details = "Skipping #{fixture_id}: #{reason}. #{inspect(error)}"
        details = if notes, do: "#{details} Notes: #{notes}", else: details
        IO.warn(details)
        details

      {_, _, true} ->
        reason = "requires #{Enum.join(requirements, ", ")}"
        details = "Skipping #{fixture_id}: #{reason}. #{inspect(error)}"
        details = if notes, do: "#{details} Notes: #{notes}", else: details
        IO.warn(details)
        details

      _ ->
        nil
    end
  end

  def run_fixture(fixture_id, relative_path, config_hash, opts \\ []) do
    requirements = Keyword.get(opts, :requirements, [])
    notes = Keyword.get(opts, :notes, nil)
    skip_if_missing = Keyword.get(opts, :skip_if_missing, true)
    run_fixture_with_method(fixture_id, relative_path, config_hash, :sync, :file,
      requirements: requirements, notes: notes, skip_if_missing: skip_if_missing
    )
  end

  def run_fixture_with_method(fixture_id, relative_path, config_hash, method, input_type, opts \\ []) do
    requirements = Keyword.get(opts, :requirements, [])
    notes = Keyword.get(opts, :notes, nil)
    skip_if_missing = Keyword.get(opts, :skip_if_missing, true)
    document_path = resolve_document(relative_path)

    if skip_if_missing and not File.exists?(document_path) do
      IO.warn("Skipping #{fixture_id}: missing document at #{document_path}")
      {:skipped, "missing document"}
    else
      config = build_config(config_hash)

      try do
        case perform_extraction(document_path, config, method, input_type) do
          {:ok, result} ->
            {:ok, result}

          {:error, reason} ->
            error = %RuntimeError{message: to_string(reason)}

            case skip_reason_for(error, fixture_id, requirements, notes) do
              nil -> {:error, reason}
              skip_msg -> {:skipped, skip_msg}
            end
        end
      rescue
        e ->
          case skip_reason_for(e, fixture_id, requirements, notes) do
            nil -> reraise e, __STACKTRACE__
            skip_msg -> {:skipped, skip_msg}
          end
      end
    end
  end

  defp perform_extraction(document_path, config, method, input_type) do
    mime_type = detect_mime_type(document_path)

    case {method, input_type} do
      {:sync, :file} ->
        Kreuzberg.extract_file(document_path, mime_type, config)

      {:sync, :bytes} ->
        bytes = File.read!(document_path)
        Kreuzberg.extract(bytes, mime_type, config)

      {:async, :file} ->
        case Kreuzberg.extract_file_async(document_path, mime_type, config) do
          {:ok, task} -> Task.await(task, :infinity)
          error -> error
        end

      {:async, :bytes} ->
        bytes = File.read!(document_path)

        case Kreuzberg.extract_async(bytes, mime_type, config) do
          {:ok, task} -> Task.await(task, :infinity)
          error -> error
        end

      {:batch_sync, :file} ->
        case Kreuzberg.batch_extract_files([document_path], mime_type, config) do
          {:ok, [first | _]} -> {:ok, first}
          {:ok, []} -> {:error, "No results from batch extraction"}
          error -> error
        end

      {:batch_sync, :bytes} ->
        bytes = File.read!(document_path)

        case Kreuzberg.batch_extract_bytes([bytes], [mime_type], config) do
          {:ok, [first | _]} -> {:ok, first}
          {:ok, []} -> {:error, "No results from batch extraction"}
          error -> error
        end

      {:batch_async, :file} ->
        case Kreuzberg.batch_extract_files_async([document_path], mime_type, config) do
          {:ok, task} ->
            case Task.await(task, :infinity) do
              {:ok, [first | _]} -> {:ok, first}
              {:ok, []} -> {:error, "No results from batch extraction"}
              error -> error
            end

          error ->
            error
        end

      {:batch_async, :bytes} ->
        bytes = File.read!(document_path)

        case Kreuzberg.batch_extract_bytes_async([bytes], [mime_type], config) do
          {:ok, task} ->
            case Task.await(task, :infinity) do
              {:ok, [first | _]} -> {:ok, first}
              {:ok, []} -> {:error, "No results from batch extraction"}
              error -> error
            end

          error ->
            error
        end

      _ ->
        {:error, "Unknown extraction method/input_type combo: #{method}/#{input_type}"}
    end
  end

  defp detect_mime_type(document_path) do
    case Kreuzberg.detect_mime_type_from_path(document_path) do
      {:ok, mime} -> mime
      _ -> nil
    end
  end

  # Assertion helpers - all return result for piping

  def assert_expected_mime(result, expected) do
    if Enum.empty?(expected) do
      result
    else
      mime = result.mime_type || ""
      if Enum.any?(expected, fn token -> String.contains?(mime, token) end) do
        result
      else
        flunk("MIME type '#{mime}' does not match expected: #{inspect(expected)}")
      end
    end
  end

  def assert_min_content_length(result, minimum) do
    content_len = String.length(result.content || "")
    if content_len >= minimum do
      result
    else
      flunk("Content length #{content_len} is less than minimum #{minimum}")
    end
  end

  def assert_max_content_length(result, maximum) do
    content_len = String.length(result.content || "")
    if content_len <= maximum do
      result
    else
      flunk("Content length #{content_len} exceeds maximum #{maximum}")
    end
  end

  def assert_content_contains_any(result, snippets) do
    if Enum.empty?(snippets) do
      result
    else
      lowered = String.downcase(result.content || "")
      if Enum.any?(snippets, fn snippet -> String.contains?(lowered, String.downcase(snippet)) end) do
        result
      else
        flunk("Content does not contain any of: #{inspect(snippets)}")
      end
    end
  end

  def assert_content_contains_all(result, snippets) do
    if Enum.empty?(snippets) do
      result
    else
      lowered = String.downcase(result.content || "")
      if Enum.all?(snippets, fn snippet -> String.contains?(lowered, String.downcase(snippet)) end) do
        result
      else
        flunk("Content does not contain all of: #{inspect(snippets)}")
      end
    end
  end

  def assert_table_count(result, min_count, max_count) do
    tables = result.tables || []
    tables_len = length(tables)

    if min_count && tables_len < min_count do
      flunk("Table count #{tables_len} is less than minimum #{min_count}")
    end

    if max_count && tables_len > max_count do
      flunk("Table count #{tables_len} exceeds maximum #{max_count}")
    end

    result
  end

  def assert_detected_languages(result, expected, min_confidence) do
    if Enum.empty?(expected) do
      result
    else
      languages = result.detected_languages || []

      if !Enum.all?(expected, fn lang -> Enum.member?(languages, lang) end) do
        flunk("Detected languages #{inspect(languages)} do not include all of #{inspect(expected)}")
      end

      if min_confidence do
        metadata = result.metadata || %{}
        confidence = metadata["confidence"] || metadata[:confidence]

        if confidence && confidence < min_confidence do
          flunk("Language confidence #{confidence} is less than minimum #{min_confidence}")
        end
      end

      result
    end
  end

  def assert_metadata_expectation(result, path, expectation) do
    metadata = result.metadata || %{}
    value = fetch_metadata_value(metadata, path)

    if value == nil do
      flunk("Metadata path '#{path}' missing in #{inspect(metadata)}")
    end

    case expectation do
      expectation when is_map(expectation) ->
        if Map.has_key?(expectation, :eq) do
          expected_val = Map.get(expectation, :eq)
          if !values_equal?(value, expected_val) do
            flunk("Metadata path '#{path}' value #{inspect(value)} != #{inspect(expected_val)}")
          end
        end

        if Map.has_key?(expectation, :gte) do
          expected_val = Map.get(expectation, :gte)
          if convert_numeric(value) < convert_numeric(expected_val) do
            flunk("Metadata path '#{path}' value #{inspect(value)} < #{inspect(expected_val)}")
          end
        end

        if Map.has_key?(expectation, :lte) do
          expected_val = Map.get(expectation, :lte)
          if convert_numeric(value) > convert_numeric(expected_val) do
            flunk("Metadata path '#{path}' value #{inspect(value)} > #{inspect(expected_val)}")
          end
        end

        if Map.has_key?(expectation, :contains) do
          contains_val = Map.get(expectation, :contains)

          cond do
            is_binary(value) && is_binary(contains_val) ->
              if !String.contains?(value, contains_val) do
                flunk("Metadata path '#{path}' value does not contain '#{contains_val}'")
              end

            is_list(value) && is_binary(contains_val) ->
              if !Enum.member?(value, contains_val) do
                flunk("Metadata path '#{path}' value does not contain '#{contains_val}'")
              end

            is_list(value) && is_list(contains_val) ->
              if !Enum.all?(contains_val, fn item -> Enum.member?(value, item) end) do
                flunk("Metadata path '#{path}' value does not contain all of #{inspect(contains_val)}")
              end

            true ->
              flunk("Unsupported contains expectation for path '#{path}'")
          end
        end

      _ ->
        if !values_equal?(value, expectation) do
          flunk("Metadata path '#{path}' value #{inspect(value)} != #{inspect(expectation)}")
        end
    end

    result
  end

  def assert_chunks(result, opts) do
    chunks = result.chunks || []
    chunks_len = length(chunks)

    if opts[:min_count] && chunks_len < opts[:min_count] do
      flunk("Chunk count #{chunks_len} is less than minimum #{opts[:min_count]}")
    end

    if opts[:max_count] && chunks_len > opts[:max_count] do
      flunk("Chunk count #{chunks_len} exceeds maximum #{opts[:max_count]}")
    end

    if opts[:each_has_content] do
      if !Enum.all?(chunks, fn chunk -> chunk.content && String.length(chunk.content) > 0 end) do
        flunk("Not all chunks have content")
      end
    end

    if opts[:each_has_embedding] do
      if !Enum.all?(chunks, fn chunk -> chunk.embedding end) do
        flunk("Not all chunks have embeddings")
      end
    end

    result
  end

  def assert_images(result, opts) do
    images = result.images || []
    images_len = length(images)

    if opts[:min_count] && images_len < opts[:min_count] do
      flunk("Image count #{images_len} is less than minimum #{opts[:min_count]}")
    end

    if opts[:max_count] && images_len > opts[:max_count] do
      flunk("Image count #{images_len} exceeds maximum #{opts[:max_count]}")
    end

    if opts[:formats_include] do
      found_formats = images |> Enum.map(fn img -> img.format end) |> Enum.uniq()

      if !Enum.all?(opts[:formats_include], fn fmt -> Enum.member?(found_formats, fmt) end) do
        flunk("Image formats #{inspect(found_formats)} do not include all of #{inspect(opts[:formats_include])}")
      end
    end

    result
  end

  def assert_pages(result, opts) do
    pages = result.pages || []
    pages_len = length(pages)

    if opts[:min_count] && pages_len < opts[:min_count] do
      flunk("Page count #{pages_len} is less than minimum #{opts[:min_count]}")
    end

    if opts[:exact_count] && pages_len != opts[:exact_count] do
      flunk("Page count #{pages_len} != exact count #{opts[:exact_count]}")
    end

    result
  end

  def assert_elements(result, opts) do
    elements = result.elements || []
    elements_len = length(elements)

    if opts[:min_count] && elements_len < opts[:min_count] do
      flunk("Element count #{elements_len} is less than minimum #{opts[:min_count]}")
    end

    if opts[:types_include] do
      found_types = elements |> Enum.map(fn elem -> elem.type end) |> Enum.uniq()

      if !Enum.all?(opts[:types_include], fn t -> Enum.member?(found_types, t) end) do
        flunk("Element types #{inspect(found_types)} do not include all of #{inspect(opts[:types_include])}")
      end
    end

    result
  end

  def assert_ocr_elements(result, opts) do
    elements = result.elements || []

    if opts[:has_elements] do
      if Enum.empty?(elements) do
        flunk("Expected elements to be present but found none")
      end
    end

    if opts[:elements_have_geometry] do
      if !Enum.all?(elements, fn elem -> elem.geometry && elem.geometry.bounding_box end) do
        flunk("Not all elements have geometry/bounding_box")
      end
    end

    if opts[:elements_have_confidence] do
      if !Enum.all?(elements, fn elem -> elem.confidence && is_number(elem.confidence) end) do
        flunk("Not all elements have valid confidence score")
      end
    end

    result
  end

  # Private helpers

  defp fetch_metadata_value(metadata, path) do
    value = lookup_metadata_path(metadata, path)

    if value == nil do
      format = metadata["format"] || metadata[:format]

      if is_map(format) do
        lookup_metadata_path(format, path)
      else
        nil
      end
    else
      value
    end
  end

  defp lookup_metadata_path(metadata, path) when is_map(metadata) do
    path
    |> String.split(".")
    |> Enum.reduce(metadata, fn segment, current ->
      if is_map(current) do
        current[segment] || current[String.to_atom(segment)]
      else
        nil
      end
    end)
  end

  defp lookup_metadata_path(_, _), do: nil

  defp values_equal?(lhs, rhs) when is_binary(lhs) and is_binary(rhs), do: lhs == rhs
  defp values_equal?(lhs, rhs) when is_number(lhs) and is_number(rhs) do
    convert_numeric(lhs) == convert_numeric(rhs)
  end
  defp values_equal?(lhs, rhs), do: lhs == rhs

  defp convert_numeric(value) when is_number(value), do: value
  defp convert_numeric(value) when is_binary(value) do
    case Float.parse(value) do
      {num, ""} -> num
      _ -> 0.0
    end
  end
  defp convert_numeric(_), do: 0.0
end
